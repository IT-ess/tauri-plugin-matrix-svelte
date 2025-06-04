use tauri::{AppHandle, Manager, Runtime, State};

use matrix_sdk::{sliding_sync::VersionBuilder, Client};

use std::path::PathBuf;

use rand::{distr::Alphanumeric, rng, Rng};
use serde::{Deserialize, Serialize};

use crate::{
    stronghold::{self, utils::BytesDto, StrongholdCollection},
    utils::fs::get_app_dir_or_create_it,
};

use super::{session::ClientSession, singletons::CLIENT};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MatrixClientConfig {
    username: String,
    password: String,
    homeserver_url: String,
    client_name: String,
}

impl MatrixClientConfig {
    pub fn new(
        username: String,
        password: String,
        homeserver_url: String,
        client_name: String,
    ) -> Self {
        MatrixClientConfig {
            username,
            password,
            homeserver_url,
            client_name,
        }
    }
}

/// Details of a login request that get submitted when calling login command
pub enum LoginRequest {
    LoginByPassword(MatrixClientConfig),
}

pub fn get_stronghold_client_key() -> BytesDto {
    BytesDto::Text("matrix_session".to_string())
}

pub async fn get_client_from_new_session<R: Runtime>(
    app_handle: &AppHandle<R>,
    login_request: LoginRequest,
    snapshot_path: &PathBuf,
) -> anyhow::Result<Client> {
    let collection_state = app_handle.state::<StrongholdCollection>();
    let client_key = get_stronghold_client_key();

    let app_data_dir = get_app_dir_or_create_it(&app_handle)?;

    stronghold::client::load_stronghold_client_or_create_it(
        collection_state.clone(),
        snapshot_path.clone(),
        client_key.clone(),
    )
    .await?;

    let matrix_config = match login_request {
        LoginRequest::LoginByPassword(config) => config,
        // TODO: add new login ways
    };

    let client_initial_state = login_and_persist_session(
        &matrix_config,
        collection_state,
        &snapshot_path,
        client_key,
        app_data_dir,
    )
    .await?;

    CLIENT
        .set(client_initial_state.clone())
        .expect("BUG: CLIENT already set!");

    Ok(client_initial_state)
}

async fn login_and_persist_session(
    config: &MatrixClientConfig,
    collection_state: State<'_, StrongholdCollection>,
    snapshot_path: &PathBuf,
    client_key: BytesDto,
    app_data_dir: PathBuf,
) -> anyhow::Result<Client> {
    let (client, client_session) = build_client(config, app_data_dir).await?;

    let matrix_auth = client.matrix_auth();

    matrix_auth
        .login_username(&config.username, &config.password)
        .initial_device_display_name(&config.client_name)
        .await?;

    let user_session = matrix_auth
        .session()
        .expect("Should have session after login");

    let full = super::session::FullMatrixSession::new(client_session, user_session);
    let serialized = serde_json::to_string(&full)?;

    stronghold::store::save_store_record(
        collection_state.clone(),
        snapshot_path.clone(),
        client_key,
        "current".to_string(),
        serialized.clone().into(),
        None,
    )
    .await?;

    stronghold::client::save(collection_state.clone(), snapshot_path.clone()).await?;

    Ok(client)
}

async fn build_client(
    config: &MatrixClientConfig,
    app_data_dir: PathBuf,
) -> anyhow::Result<(Client, ClientSession)> {
    let db_subfolder: String = rng()
        .sample_iter(Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    let db_path = app_data_dir.join("matrix-db").join(db_subfolder);

    std::fs::create_dir_all(&db_path)?;

    let passphrase: String = rng()
        .sample_iter(Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let client = Client::builder()
        .homeserver_url(&config.homeserver_url)
        .sqlite_store(&db_path, Some(&passphrase))
        .sliding_sync_version_builder(VersionBuilder::DiscoverNative) // Comment this if your homeserver doesn't support simplified sliding sync.
        .handle_refresh_tokens()
        .build()
        .await?;

    let client_session = ClientSession::new(config.homeserver_url.clone(), db_path, passphrase);

    Ok((client, client_session))
}
