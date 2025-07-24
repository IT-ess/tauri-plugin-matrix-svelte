use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};

use crate::keychain::retrieve_session;

use matrix_sdk::Client;

use std::path::PathBuf;

use matrix_sdk::authentication::matrix::MatrixSession;

use super::singletons::CLIENT;

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

pub async fn get_matrix_session_option<R: Runtime>(
    app_handle: &AppHandle<R>,
) -> anyhow::Result<Option<FullMatrixSession>> {
    // let read_session = stronghold::store::get_store_record(
    //     collection_state,
    //     snapshot_path,
    //     client_key,
    //     "current".to_string(),
    // )
    // .await?;
    let session_option = retrieve_session(app_handle)?;

    match session_option {
        None => Ok(None),
        Some(session_string) => {
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
