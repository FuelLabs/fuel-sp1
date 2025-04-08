pub mod block_execution_game;
pub mod decompression_game;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// This error occurs when the input cannot be deserialized using bincode
    #[error("failed to deserialize input: `{0}`")]
    FailedToDeserializeInput(#[from] bincode::Error),
    /// This error occurs when the public outputs from the zkvm cannot be deserialized
    #[error("failed to deserialize public output: `{0}`")]
    FailedToDeserializePublicOutput(String),
    /// This error occurs when the proving game fails to execute
    #[error("failed to execute proving game: `{0}`")]
    FailedToExecuteProvingGame(String),
    /// This error occurs when the proving game fails to prove
    #[error("failed to prove proving game: `{0}`")]
    FailedToProveProvingGame(String),
    /// This error occurs when a fault/mismatch is detected
    #[error("FAULT: `{0}`")]
    Fault(String),
}

pub type Result<T> = core::result::Result<T, Error>;
