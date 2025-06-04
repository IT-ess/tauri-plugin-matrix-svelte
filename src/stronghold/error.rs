use serde::{Serialize, Serializer};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("stronghold not initialized")]
    StrongholdNotInitialized,
    #[error(transparent)]
    Stronghold(#[from] iota_stronghold::ClientError),
    #[error(transparent)]
    Memory(#[from] iota_stronghold::MemoryError),
    #[error(transparent)]
    Procedure(#[from] iota_stronghold::procedures::ProcedureError),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}
