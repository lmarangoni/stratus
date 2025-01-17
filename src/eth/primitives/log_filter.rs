//! Log Filter Module
//!
//! Facilitates filtering of Ethereum log entries based on specified criteria.
//! This is crucial for event monitoring and handling, allowing users and
//! applications to specify filters for logs based on block range, addresses,
//! and topics. The module provides the core functionality to create and apply
//! these filters, enhancing the efficiency of log querying and processing.

use crate::eth::primitives::Address;
use crate::eth::primitives::BlockNumber;
use crate::eth::primitives::LogMined;
use crate::eth::primitives::LogTopic;
use crate::ext::not;
use crate::gen_newtype_from;

#[derive(Debug)]
pub struct LogFilter {
    pub from_block: BlockNumber,
    pub to_block: Option<BlockNumber>,
    pub addresses: Vec<Address>,
    pub topics_combinations: Vec<LogFilterTopicCombination>,
}

impl LogFilter {
    /// Checks if a log matches the filter.
    pub fn matches(&self, log: &LogMined) -> bool {
        // filter block range
        if log.block_number < self.from_block {
            return false;
        }
        if self.to_block.as_ref().is_some_and(|to_block| log.block_number > *to_block) {
            return false;
        }

        // filter address
        if not(self.addresses.contains(log.address())) {
            return false;
        }

        // filter topics
        let log_topics = log.topics();
        let mut found_matching_combination = self.topics_combinations.is_empty();
        for combination in self.topics_combinations.iter() {
            if combination.matches(log_topics) {
                found_matching_combination = true;
                break;
            }
        }

        found_matching_combination
    }
}

#[derive(Debug)]
pub struct LogFilterTopicCombination(Vec<(usize, LogTopic)>);

gen_newtype_from!(self = LogFilterTopicCombination, other = Vec<(usize, LogTopic)>);

impl LogFilterTopicCombination {
    fn matches(&self, log_topics: &[LogTopic]) -> bool {
        for (topic_index, topic) in &self.0 {
            if log_topics.get(*topic_index).is_some_and(|log_topic| log_topic != topic) {
                return false;
            }
        }
        true
    }
}
