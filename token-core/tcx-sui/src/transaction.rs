/// FUNCTION: sign_tx(SignParam{input: TronTxInput}): TronTxOutput
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SuiTxInput {
    #[prost(oneof = "sui_tx_input::SuiTxType", tags = "1, 2")]
    pub sui_tx_type: ::core::option::Option<sui_tx_input::SuiTxType>,
}

pub mod sui_tx_input {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum SuiTxType {
        #[prost(message, tag = "1")]
        RawTx(super::RawTx),
        #[prost(message, tag = "2")]
        Transfer(super::NewTransfer),
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RawTx {
    #[prost(string, tag = "1")]
    pub intent: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub tx_data: ::prost::alloc::string::String,
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NewTransfer {
    #[prost(string, tag = "3")]
    pub recipient: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub sender: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "5")]
    pub gas_payment: ::core::option::Option<ProstObjectRef>,
    #[prost(uint64, tag = "6")]
    pub gas_budget: u64,
    #[prost(uint64, tag = "7")]
    pub gas_price: u64,
    #[prost(oneof = "new_transfer::TransferType", tags = "1, 2")]
    pub transfer_type: ::core::option::Option<new_transfer::TransferType>,
}

pub mod new_transfer {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum TransferType {
        #[prost(message, tag = "1")]
        Sui(super::SuiTransfer),
        #[prost(message, tag = "2")]
        Object(super::ProstObjectRef),
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SuiTransfer {
    #[prost(uint64, tag = "1")]
    pub amount: u64,
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProstObjectRef {
    #[prost(bytes, tag = "1")]
    pub object_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "2")]
    pub seq_num: u64,
    #[prost(bytes, tag = "3")]
    pub object_digest: ::prost::alloc::vec::Vec<u8>,
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SuiTxOuput {
    #[prost(string, tag = "1")]
    pub tx_data: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub signature: ::prost::alloc::string::String,
}
