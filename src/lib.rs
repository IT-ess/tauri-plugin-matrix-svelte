use tauri::{
  plugin::{Builder, TauriPlugin},
  Manager, Runtime,
};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::MatrixSvelte;
#[cfg(mobile)]
use mobile::MatrixSvelte;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the matrix-svelte APIs.
pub trait MatrixSvelteExt<R: Runtime> {
  fn matrix_svelte(&self) -> &MatrixSvelte<R>;
}

impl<R: Runtime, T: Manager<R>> crate::MatrixSvelteExt<R> for T {
  fn matrix_svelte(&self) -> &MatrixSvelte<R> {
    self.state::<MatrixSvelte<R>>().inner()
  }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("matrix-svelte")
    .invoke_handler(tauri::generate_handler![commands::ping])
    .setup(|app, api| {
      #[cfg(mobile)]
      let matrix_svelte = mobile::init(app, api)?;
      #[cfg(desktop)]
      let matrix_svelte = desktop::init(app, api)?;
      app.manage(matrix_svelte);
      Ok(())
    })
    .build()
}
