//! for things that don't belong in the cosmos or ethereum libraries but also don't belong
//! in a function specific library

use clarity::Error as ClarityError;
use deep_space::error::AddressError as CosmosAddressError;
use deep_space::error::CosmosGrpcError;
use num_bigint::ParseBigIntError;
use std::fmt;
use std::fmt::Debug;
use tonic::Status;
use web30::jsonrpc::error::Web3Error;

#[derive(Debug)]
pub enum GravityError {
    InvalidBigInt(ParseBigIntError),
    CosmosGrpcError(CosmosGrpcError),
    CosmosAddressError(CosmosAddressError),
    EthereumRestError(Web3Error),
    InvalidBridgeStateError(String),
    FailedToUpdateValset,
    EthereumContractError(String),
    InvalidOptionsError(String),
    ClarityError(ClarityError),
    TimeoutError,
    InvalidEventLogError(String),
    GravityGrpcError(Status),
    InsufficientVotingPowerToPass(String),
    ParseBigIntError(ParseBigIntError),
    ValsetUpToDate,
    ValidationError(String),
    UnrecoverableError(String),
}

impl fmt::Display for GravityError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GravityError::GravityGrpcError(val) => write!(f, "Gravity gRPC error {}", val),
            GravityError::CosmosGrpcError(val) => write!(f, "Cosmos gRPC error {}", val),
            GravityError::InvalidBigInt(val) => {
                write!(f, "Got invalid BigInt from cosmos! {}", val)
            }
            GravityError::CosmosAddressError(val) => write!(f, "Cosmos Address error {}", val),
            GravityError::EthereumRestError(val) => write!(f, "Ethereum REST error {}", val),
            GravityError::InvalidOptionsError(val) => {
                write!(f, "Invalid TX options for this call {}", val)
            }
            GravityError::InvalidBridgeStateError(val) => {
                write!(f, "Invalid bridge state! {}", val)
            }
            GravityError::FailedToUpdateValset => write!(f, "ValidatorSetUpdate Failed!"),
            GravityError::TimeoutError => write!(f, "Operation timed out!"),
            GravityError::ClarityError(val) => write!(f, "Clarity Error {}", val),
            GravityError::InvalidEventLogError(val) => write!(f, "InvalidEvent: {}", val),
            GravityError::EthereumContractError(val) => {
                write!(f, "Contract operation failed: {}", val)
            }
            GravityError::InsufficientVotingPowerToPass(val) => {
                write!(f, "{}", val)
            }
            GravityError::ParseBigIntError(val) => write!(f, "Failed to parse big integer {}", val),
            GravityError::ValsetUpToDate => {
                write!(
                    f,
                    "latest validator set is synced between Ethereum and Cosmos"
                )
            }
            GravityError::ValidationError(s) => write!(f, "Validation error: {}", s),
            GravityError::UnrecoverableError(s) => write!(f, "Unrecoverable error: {}", s),
        }
    }
}

impl From<CosmosGrpcError> for GravityError {
    fn from(error: CosmosGrpcError) -> Self {
        GravityError::CosmosGrpcError(error)
    }
}

impl From<ClarityError> for GravityError {
    fn from(error: ClarityError) -> Self {
        GravityError::InvalidEventLogError(error.to_string())
    }
}

impl From<Web3Error> for GravityError {
    fn from(error: Web3Error) -> Self {
        GravityError::EthereumRestError(error)
    }
}

impl From<Status> for GravityError {
    fn from(error: Status) -> Self {
        GravityError::GravityGrpcError(error)
    }
}

impl From<CosmosAddressError> for GravityError {
    fn from(error: CosmosAddressError) -> Self {
        GravityError::InvalidEventLogError(error.to_string())
    }
}
impl From<ParseBigIntError> for GravityError {
    fn from(error: ParseBigIntError) -> Self {
        GravityError::InvalidEventLogError(error.to_string())
    }
}
