//! Address Module
//!
//! This module handles Ethereum addresses, a fundamental component for
//! identifying accounts and contracts within the Ethereum network. The address
//! is a 20-byte identifier derived from the public key of an account and is
//! crucial for transactions and smart contract interactions. Key functionalities
//! include generating new addresses, determining special addresses like the zero
//! address (often used to represent non-specific or null addresses in smart
//! contracts) and the coinbase address (which is crucial in mining processes
//! for receiving block rewards).

use std::fmt::Display;
use std::ops::Deref;

use ethabi::Token;
use ethereum_types::H160;
use ethers_core::types::NameOrAddress;
use fake::Dummy;
use fake::Faker;
use hex_literal::hex;
use revm::primitives::Address as RevmAddress;
use sqlx::database::HasArguments;
use sqlx::database::HasValueRef;
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::Decode;

use crate::gen_newtype_from;

/// Address of an Ethereum account (wallet or contract).
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Address(H160);

impl Address {
    // Special ETH address used in some contexts.
    pub const ZERO: Address = Address(H160::zero());

    /// Special address that receives the block reward.
    pub const COINBASE: Address = Address(H160(hex!("00000000000000000000000000000000000000ff")));

    /// Creates a new address from the given bytes.
    pub const fn new(bytes: [u8; 20]) -> Self {
        Self(H160(bytes))
    }

    /// Checks if current address is the zero address.
    pub fn is_zero(&self) -> bool {
        self == &Self::ZERO
    }

    /// Checks if current address is the coinbase address.
    pub fn is_coinbase(&self) -> bool {
        self == &Self::COINBASE
    }

    /// Checks if current address should have their updates ignored.
    ///
    /// * Coinbase is ignored because we do not charge gas, otherwise it will have to be updated for every transaction.
    /// * Not sure if zero address should be ignored or not.
    pub fn is_ignored(&self) -> bool {
        self.is_coinbase() || self.is_zero()
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

impl Dummy<Faker> for Address {
    fn dummy_with_rng<R: ethers_core::rand::prelude::Rng + ?Sized>(_: &Faker, rng: &mut R) -> Self {
        H160::random_using(rng).into()
    }
}

impl Deref for Address {
    type Target = H160;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// -----------------------------------------------------------------------------
// Conversions: Other -> Self
// -----------------------------------------------------------------------------
gen_newtype_from!(self = Address, other = H160, [u8; 20]);

impl From<RevmAddress> for Address {
    fn from(value: RevmAddress) -> Self {
        Address(value.0 .0.into())
    }
}

impl From<NameOrAddress> for Address {
    fn from(value: NameOrAddress) -> Self {
        match value {
            NameOrAddress::Name(_) => panic!("TODO"),
            NameOrAddress::Address(value) => Self(value),
        }
    }
}

// -----------------------------------------------------------------------------
// Conversions: sqlx -> Self
// -----------------------------------------------------------------------------
impl<'r> sqlx::Decode<'r, sqlx::Postgres> for Address {
    fn decode(value: <sqlx::Postgres as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        let value = <[u8; 20] as Decode<sqlx::Postgres>>::decode(value)?;
        Ok(value.into())
    }
}

impl sqlx::Type<sqlx::Postgres> for Address {
    fn type_info() -> <sqlx::Postgres as sqlx::Database>::TypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("BYTEA")
    }
}

impl AsRef<[u8]> for Address {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

// -----------------------------------------------------------------------------
// Conversions: Self -> Other
// -----------------------------------------------------------------------------
impl From<Address> for H160 {
    fn from(value: Address) -> Self {
        value.0
    }
}

impl From<Address> for RevmAddress {
    fn from(value: Address) -> Self {
        RevmAddress(value.0 .0.into())
    }
}

impl From<Address> for Token {
    fn from(value: Address) -> Self {
        Token::Address(value.0)
    }
}

impl From<Address> for [u8; 20] {
    fn from(value: Address) -> Self {
        H160::from(value.clone()).0
    }
}

// -----------------------------------------------------------------------------
// Conversions: Self -> sqlx
// -----------------------------------------------------------------------------
impl<'q> sqlx::Encode<'q, sqlx::Postgres> for Address {
    fn encode_by_ref(&self, buf: &mut <sqlx::Postgres as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        self.0 .0.encode(buf)
    }
}
