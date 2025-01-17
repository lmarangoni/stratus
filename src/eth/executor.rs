//! EthExecutor: Ethereum Transaction Coordinator
//!
//! This module provides the `EthExecutor` struct, which acts as a coordinator for executing Ethereum transactions.
//! It encapsulates the logic for transaction execution, state mutation, and event notification.
//! `EthExecutor` is designed to work with the `Evm` trait implementations to execute transactions and calls,
//! while also interfacing with a miner component to handle block mining and a storage component to persist state changes.

use std::sync::Arc;
use std::thread;

use anyhow::anyhow;
use nonempty::NonEmpty;
use tokio::runtime::Handle;
use tokio::sync::broadcast;
use tokio::sync::oneshot;
use tokio::sync::Mutex;

use crate::eth::evm::Evm;
use crate::eth::evm::EvmInput;
use crate::eth::miner::BlockMiner;
use crate::eth::primitives::Block;
use crate::eth::primitives::CallInput;
use crate::eth::primitives::Execution;
use crate::eth::primitives::LogMined;
use crate::eth::primitives::StoragePointInTime;
use crate::eth::primitives::TransactionInput;
use crate::eth::storage::EthStorage;
use crate::eth::storage::EthStorageError;

/// Number of events in the backlog.
const NOTIFIER_CAPACITY: usize = u16::MAX as usize;

type EvmTask = (EvmInput, oneshot::Sender<anyhow::Result<Execution>>);

/// The EthExecutor struct is responsible for orchestrating the execution of Ethereum transactions.
/// It holds references to the EVM, block miner, and storage, managing the overall process of
/// transaction execution, block production, and state management.
pub struct EthExecutor {
    // Channel to send transactions to background EVMs.
    evm_tx: crossbeam_channel::Sender<EvmTask>,

    // Mutex-wrapped miner for creating new blockchain blocks.
    miner: Mutex<BlockMiner>,

    // Shared storage backend for persisting blockchain state.
    eth_storage: Arc<dyn EthStorage>,

    // Broadcast channels for notifying subscribers about new blocks and logs.
    block_notifier: broadcast::Sender<Block>,
    log_notifier: broadcast::Sender<LogMined>,
}

impl EthExecutor {
    /// Creates a new executor.
    pub fn new(evms: NonEmpty<Box<dyn Evm>>, eth_storage: Arc<dyn EthStorage>) -> Self {
        let evm_tx = spawn_background_evms(evms);

        Self {
            evm_tx,
            miner: Mutex::new(BlockMiner::new(Arc::clone(&eth_storage))),
            eth_storage,
            block_notifier: broadcast::channel(NOTIFIER_CAPACITY).0,
            log_notifier: broadcast::channel(NOTIFIER_CAPACITY).0,
        }
    }

    /// Executes Ethereum transactions and facilitates block creation.
    ///
    /// This function is a key part of the transaction processing pipeline. It begins by validating
    /// incoming transactions and then proceeds to execute them. Unlike conventional blockchain systems,
    /// the block creation here is not dictated by timed intervals but is instead triggered by transaction
    /// processing itself. This method encapsulates the execution, block mining, and state mutation,
    /// concluding with broadcasting necessary notifications for the newly created block and associated transaction logs.
    ///
    /// TODO: too much cloning that can be optimized here.
    pub async fn transact(&self, transaction: TransactionInput) -> anyhow::Result<Execution> {
        tracing::info!(
            hash = %transaction.hash,
            nonce = %transaction.nonce,
            from = %transaction.from,
            signer = %transaction.signer,
            to = ?transaction.to,
            data_len = %transaction.input.len(),
            data = %transaction.input,
            "executing real transaction"
        );

        // validate
        if transaction.signer.is_zero() {
            tracing::warn!("rejecting transaction from zero address");
            return Err(anyhow!("Transaction sent from zero address is not allowed."));
        }

        // execute transaction until no more conflicts
        // todo: must have a stop condition like timeout or max number of retries
        let (execution, block) = loop {
            // execute and check conflicts before mining block
            let execution = self.execute_in_evm(transaction.clone().try_into()?).await?;
            if let Some(conflicts) = self.eth_storage.check_conflicts(&execution).await? {
                tracing::warn!(?conflicts, "storage conflict detected before mining block");
                continue;
            }

            // mine and save block
            let mut miner_lock = self.miner.lock().await;
            let block = miner_lock.mine_with_one_transaction(transaction.clone(), execution.clone()).await?;
            match self.eth_storage.save_block(block.clone()).await {
                Ok(()) => {}
                Err(EthStorageError::Conflict(conflicts)) => {
                    tracing::warn!(?conflicts, "storage conflict detected when saving block");
                    continue;
                }
                Err(e) => return Err(e.into()),
            };
            break (execution, block);
        };

        // notify new blocks
        if let Err(e) = self.block_notifier.send(block.clone()) {
            tracing::error!(reason = ?e, "failed to send block notification");
        };

        // notify transaction logs
        for trx in block.transactions {
            for log in trx.logs {
                if let Err(e) = self.log_notifier.send(log) {
                    tracing::error!(reason = ?e, "failed to send log notification");
                };
            }
        }

        Ok(execution)
    }

    /// Execute a function and return the function output. State changes are ignored.
    pub async fn call(&self, input: CallInput, point_in_time: StoragePointInTime) -> anyhow::Result<Execution> {
        tracing::info!(
            from = %input.from,
            to = ?input.to,
            data_len = input.data.len(),
            data = %input.data,
            "executing read-only transaction"
        );

        let execution = self.execute_in_evm((input, point_in_time).into()).await?;
        Ok(execution)
    }

    /// Submits a transaction to the EVM and awaits for its execution.
    async fn execute_in_evm(&self, evm_input: EvmInput) -> anyhow::Result<Execution> {
        let (execution_tx, execution_rx) = oneshot::channel::<anyhow::Result<Execution>>();
        self.evm_tx.send((evm_input, execution_tx))?;
        execution_rx.await?
    }

    /// Subscribe to new blocks events.
    pub fn subscribe_to_new_heads(&self) -> broadcast::Receiver<Block> {
        self.block_notifier.subscribe()
    }

    /// Subscribe to new logs events.
    pub fn subscribe_to_logs(&self) -> broadcast::Receiver<LogMined> {
        self.log_notifier.subscribe()
    }
}

// for each evm, spawn a new thread that runs in an infinite loop executing transactions.
fn spawn_background_evms(evms: NonEmpty<Box<dyn Evm>>) -> crossbeam_channel::Sender<EvmTask> {
    let (evm_tx, evm_rx) = crossbeam_channel::unbounded::<EvmTask>();

    for mut evm in evms {
        // clone shared resources for thread
        let evm_rx = evm_rx.clone();
        let tokio = Handle::current();

        // prepare thread
        let t = thread::Builder::new().name("evm".into());
        t.spawn(move || {
            // make tokio runtime available to this thread
            let _tokio_guard = tokio.enter();

            // keep executing transactions until the channel is closed
            while let Ok((input, tx)) = evm_rx.recv() {
                if let Err(e) = tx.send(evm.execute(input)) {
                    tracing::error!(reason = ?e, "failed to send evm execution result");
                };
            }
            tracing::warn!("stopping evm thread because task channel was closed");
        })
        .expect("spawning evm threads should not fail");
    }
    evm_tx
}
