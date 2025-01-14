//! for things that don't belong in the cosmos or ethereum libraries but also don't belong
//! in a function specific library

use std::fmt::Debug;

use clarity::Error as ClarityError;
use deep_space::error::{AddressError as CosmosAddressError, CosmosGrpcError};
use tonic::Status;
use u64_array_bigints::FromStrRadixErr;
use web30::jsonrpc::error::Web3Error;

#[derive(thiserror::Error, Debug)]
pub enum GravityError {
    #[error("{0}")]
    ValidationError(String),

    #[error("{0}")]
    RecoverableError(String),

    #[error("{0}")]
    UnrecoverableError(String),

    // we can pass String info here as well if we need more context/details
    #[error(transparent)]
    RpcError(#[from] Box<dyn std::error::Error + Send>),
}

impl From<CosmosGrpcError> for GravityError {
    fn from(error: CosmosGrpcError) -> Self {
        GravityError::RpcError(Box::new(error))
    }
}

impl From<ClarityError> for GravityError {
    fn from(error: ClarityError) -> Self {
        GravityError::ValidationError(error.to_string())
    }
}

impl From<Web3Error> for GravityError {
    fn from(error: Web3Error) -> Self {
        GravityError::RpcError(Box::new(error))
    }
}

impl From<Status> for GravityError {
    fn from(error: Status) -> Self {
        GravityError::RpcError(Box::new(error))
    }
}

impl From<CosmosAddressError> for GravityError {
    fn from(error: CosmosAddressError) -> Self {
        GravityError::ValidationError(error.to_string())
    }
}
impl From<FromStrRadixErr> for GravityError {
    fn from(error: FromStrRadixErr) -> Self {
        GravityError::ValidationError(error.to_string())
    }
}
