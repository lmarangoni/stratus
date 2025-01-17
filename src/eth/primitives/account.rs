//! Account Module
//!
//! The Account module is central to Ethereum's functionality, representing
//! both user wallets and contracts. It encapsulates key aspects of an Ethereum
//! account, such as its unique address, nonce (which tracks the number of
//! transactions sent from the account), current balance, and in the case of
//! smart contracts, their associated bytecode. This module is pivotal for
//! tracking account states and differentiating between standard accounts and
//! contract accounts.

use revm::primitives::AccountInfo as RevmAccountInfo;
use revm::primitives::Address as RevmAddress;
use revm::primitives::KECCAK_EMPTY;

use crate::eth::primitives::Address;
use crate::eth::primitives::Bytes;
use crate::eth::primitives::Nonce;
use crate::eth::primitives::Wei;
use crate::ext::OptionExt;

/// Ethereum account (wallet or contract).
#[derive(Debug, Clone, Default)]
pub struct Account {
    /// Immutable address of the account.
    pub address: Address,

    /// Current nonce of the account. Changes every time a transaction is sent.
    pub nonce: Nonce,

    /// Current balance of the account. Changes when a transfer is made or the account pays a fee for executing a transaction.
    pub balance: Wei,

    /// Contract bytecode. Present only if the account is a contract.
    pub bytecode: Option<Bytes>,
}

impl Account {
    /// Checks the current account is a contract.
    pub fn is_contract(&self) -> bool {
        match self.bytecode {
            Some(ref bytecode) => !bytecode.is_empty(),
            None => false,
        }
    }
}

// -----------------------------------------------------------------------------
// Conversions: Other -> Self
// -----------------------------------------------------------------------------
impl From<(RevmAddress, RevmAccountInfo)> for Account {
    fn from(value: (RevmAddress, RevmAccountInfo)) -> Self {
        Self {
            address: value.0.into(),
            nonce: value.1.nonce.into(),
            balance: value.1.balance.into(),
            bytecode: value.1.code.map_into(),
        }
    }
}

// -----------------------------------------------------------------------------
// Conversions: Self -> Other
// -----------------------------------------------------------------------------
impl From<Account> for RevmAccountInfo {
    fn from(value: Account) -> Self {
        Self {
            nonce: value.nonce.into(),
            balance: value.balance.into(),
            code_hash: KECCAK_EMPTY,
            code: value.bytecode.map_into(),
        }
    }
}
