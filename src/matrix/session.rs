use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime, State};

use crate::stronghold::{utils::BytesDto, StrongholdCollection};

use matrix_sdk::Client;

use std::path::PathBuf;

use matrix_sdk::authentication::matrix::MatrixSession;

use crate::stronghold::{self};

use super::{login::get_stronghold_client_key, singletons::CLIENT};

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientSession {
    homeserver: String,
    db_path: PathBuf,
    passphrase: String,
}

impl ClientSession {
    pub fn new(homeserver: String, db_path: PathBuf, passphrase: String) -> Self {
        ClientSession {
            homeserver,
            db_path,
            passphrase,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FullMatrixSession {
    client_session: ClientSession,
    user_session: MatrixSession,
}

impl FullMatrixSession {
    pub fn new(client_session: ClientSession, user_session: MatrixSession) -> Self {
        FullMatrixSession {
            client_session,
            user_session,
        }
    }
}

pub async fn get_matrix_session_option(
    collection_state: State<'_, StrongholdCollection>,
    snapshot_path: PathBuf,
    client_key: BytesDto,
) -> anyhow::Result<Option<FullMatrixSession>> {
    let read_session = stronghold::store::get_store_record(
        collection_state,
        snapshot_path,
        client_key,
        "current".to_string(),
    )
    .await?;

    match read_session {
        None => Ok(None),
        Some(raw_session) => {
            let session_string = String::from_utf8_lossy(&raw_session).into_owned();

            let session: FullMatrixSession = serde_json::from_str(&session_string)?;
            Ok(Some(session))
        }
    }
}

pub async fn restore_client_from_session(session: FullMatrixSession) -> anyhow::Result<Client> {
    let FullMatrixSession {
        client_session,
        user_session,
    } = session;

    let client = Client::builder()
        .homeserver_url(client_session.homeserver)
        .sqlite_store(client_session.db_path, Some(&client_session.passphrase))
        .build()
        .await?;

    client.restore_session(user_session).await?;

    CLIENT
        .set(client.clone())
        .expect("BUG: CLIENT already set!");

    Ok(client)
}

pub async fn try_get_session<R: Runtime>(
    app_handle: &AppHandle<R>,
    snapshot_path: PathBuf,
) -> anyhow::Result<Option<FullMatrixSession>> {
    let collection_state = app_handle.state::<StrongholdCollection>();
    let client_key = get_stronghold_client_key();

    stronghold::client::load_stronghold_client_or_create_it(
        collection_state.clone(),
        snapshot_path.clone(),
        client_key.clone(),
    )
    .await?;

    let session_option = get_matrix_session_option(
        collection_state.clone(),
        snapshot_path.clone(),
        client_key.clone(),
    )
    .await?;

    Ok(session_option)
}
