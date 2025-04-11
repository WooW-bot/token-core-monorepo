pub mod address;
mod primitives;
pub mod signer;
mod sui_serde;
pub mod transaction;

pub use transaction::{
    new_transfer::TransferType, sui_tx_input::SuiTxType, NewTransfer, ProstObjectRef, RawTx,
    SuiTransfer, SuiTxInput, SuiTxOuput,
};

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("sui address parse error")]
    AddressParseError,
    #[error("sui tx intent base64 parse error")]
    IntentBs64ParseError,
    #[error("sui tx intent bcs parse error")]
    IntentBcsParseError,
    #[error("sui tx data base64 parse error")]
    TxDataBase64ParseError,
    #[error("sui tx data bcs parse error")]
    TxDataBcsParseError,
    #[error("sui account not found")]
    CannotFoundSuiAccount,
    #[error("sui curve type is invalid")]
    InvalidSuiCurveType,
    #[error("bcs serialize failed")]
    BcsSerializeFailed,
    #[error("invalid object id length")]
    InvalidObjectID,
    #[error("invalid object digest length")]
    InvalidObjectDigest,
    #[error("tx must be 'raw' or 'transfer'")]
    EmptyTxType,
    #[error("transfer type must be 'sui' or 'object'")]
    EmptyTransferType,
    #[error("objectRef should not empty")]
    EmptyObjectRef,
}

pub mod sui {
    pub const CHAINS: [&str; 2] = ["SUI", "ONECHAIN"];

    pub type Address = crate::address::SuiAddress;
    pub type TransactionInput = crate::transaction::SuiTxInput;
    pub type TransactionOutput = crate::transaction::SuiTxOuput;
}
