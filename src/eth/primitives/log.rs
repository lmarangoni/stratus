//! Log Module
//!
//! Handles Ethereum log entries, which are key components of Ethereum's event
//! system. Logs are emitted by smart contracts and are crucial for tracking
//! contract events. This module defines the structure of logs, including the
//! emitting address, topics, and additional data. It also provides conversion
//! functions to translate between internal and external log representations.

use itertools::Itertools;
use revm::primitives::Log as RevmLog;

use crate::eth::primitives::Address;
use crate::eth::primitives::Bytes;
use crate::eth::primitives::LogTopic;

/// Log is an event emitted by the EVM during contract execution.
#[derive(Debug, Clone, PartialEq, Eq, fake::Dummy, serde::Serialize, serde::Deserialize)]
pub struct Log {
    /// Address that emitted the log.
    pub address: Address,

    /// Topics (0 to 4 positions) describing the log.
    pub topics: Vec<LogTopic>,

    /// Additional data.
    pub data: Bytes,
}

// -----------------------------------------------------------------------------
// Conversions: Other -> Self
// ----------------------------------------------------------------------------
impl From<RevmLog> for Log {
    fn from(value: RevmLog) -> Self {
        Self {
            address: value.address.into(),
            topics: value.topics.into_iter().map_into().collect(),
            data: value.data.into(),
        }
    }
}
