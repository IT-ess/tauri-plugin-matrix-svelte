use tauri::{command, AppHandle, Runtime};

use crate::matrix::login::MatrixClientConfig;
use crate::matrix::requests::MatrixRequest;
use crate::ping::PingRequest;
use crate::ping::PingResponse;
use crate::MatrixSvelteExt;
use crate::Result;

#[command]
pub(crate) async fn ping<R: Runtime>(
    app: AppHandle<R>,
    payload: PingRequest,
) -> Result<PingResponse> {
    app.matrix_svelte().ping(payload)
}

#[command]
pub(crate) async fn login_and_create_new_session<R: Runtime>(
    app: AppHandle<R>,
    config: MatrixClientConfig,
) -> Result<()> {
    // TODO: create error variants to display rich error messages to frontend
    app.matrix_svelte()
        .login_and_create_new_session(config)
        .await
}

#[command]
pub(crate) fn submit_async_request<R: Runtime>(
    app: AppHandle<R>,
    request: MatrixRequest,
) -> Result<()> {
    // TODO: create add some sender / receiver mechanisme to display rich error
    // messages to frontend in case of error
    app.matrix_svelte().submit_async_request(request)
}
