use std::{path::PathBuf, time::Duration};

use super::client::get_client;
use super::{StrongholdCollection, utils::BytesDto};
use tauri::State;

use super::error::Result;

pub async fn get_store_record(
    collection: State<'_, StrongholdCollection>,
    snapshot_path: PathBuf,
    client: BytesDto,
    key: String,
) -> Result<Option<Vec<u8>>> {
    let client = get_client(collection, snapshot_path, client)?;
    client.store().get(key.as_ref()).map_err(Into::into)
}

pub async fn save_store_record(
    collection: State<'_, StrongholdCollection>,
    snapshot_path: PathBuf,
    client: BytesDto,
    key: String,
    value: Vec<u8>,
    lifetime: Option<Duration>,
) -> Result<Option<Vec<u8>>> {
    let client = get_client(collection, snapshot_path, client)?;
    client
        .store()
        .insert(key.as_bytes().to_vec(), value, lifetime)
        .map_err(Into::into)
}

pub async fn _remove_store_record(
    collection: State<'_, StrongholdCollection>,
    snapshot_path: PathBuf,
    client: BytesDto,
    key: String,
) -> Result<Option<Vec<u8>>> {
    let client = get_client(collection, snapshot_path, client)?;
    client.store().delete(key.as_ref()).map_err(Into::into)
}
