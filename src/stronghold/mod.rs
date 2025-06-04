// Code heavily inspired by the original Tauri Stronghold implementation available [here](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/stronghold)

//! Store secrets and keys using the [IOTA Stronghold](https://github.com/iotaledger/stronghold.rs) encrypted database and secure runtime.

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use client::Stronghold;
use tauri::{AppHandle, Manager, Runtime};

use crate::utils::{config::get_plugin_config, fs::get_app_dir_or_create_it};

pub mod builder;
pub mod client;
pub mod error;
pub mod store;
pub mod utils;

#[derive(Default)]
pub struct StrongholdCollection(Arc<Mutex<HashMap<PathBuf, Stronghold>>>);

pub struct SnapshotPath(pub PathBuf);

pub fn init_stronghold_client<R: Runtime>(app_handle: &AppHandle<R>) -> crate::Result<()> {
    let plugin_config = get_plugin_config(&app_handle)?;

    let app_dir = get_app_dir_or_create_it(&app_handle)?;

    // stronghold config
    let salt_path = app_dir.join("salt");
    let snapshot_path = app_dir.join("matrix.stronghold");

    let stronghold = builder::Builder::with_blake2(&salt_path);
    stronghold.build_and_init(
        &app_handle,
        snapshot_path.clone(),
        plugin_config.stronghold_password,
    )?;

    app_handle.manage(SnapshotPath(snapshot_path));
    Ok(())
}
