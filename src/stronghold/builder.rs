use std::path::PathBuf;

use super::utils::KeyDerivation;
use super::{StrongholdCollection, client::Stronghold};
use tauri::{AppHandle, Manager, Runtime, State};

use super::error::Result;
use zeroize::Zeroize;

type PasswordHashFn = dyn Fn(&str) -> Vec<u8> + Send + Sync;

pub struct PasswordHashFunction(Box<PasswordHashFn>);

enum PasswordHashFunctionKind {
    _Argon2(PathBuf),
    Blake2(PathBuf),
    _Custom(Box<PasswordHashFn>),
}

pub fn initialize(
    collection: State<'_, StrongholdCollection>,
    hash_function: State<'_, PasswordHashFunction>,
    snapshot_path: PathBuf,
    mut password: String,
) -> Result<()> {
    let hash = (hash_function.0)(&password);
    password.zeroize();
    let stronghold = Stronghold::new(snapshot_path.clone(), hash)?;

    collection
        .0
        .lock()
        .unwrap()
        .insert(snapshot_path, stronghold);

    Ok(())
}

pub struct Builder {
    password_hash_function: PasswordHashFunctionKind,
}

impl Builder {
    pub fn _new<F: Fn(&str) -> Vec<u8> + Send + Sync + 'static>(password_hash_function: F) -> Self {
        Self {
            password_hash_function: PasswordHashFunctionKind::_Custom(Box::new(
                password_hash_function,
            )),
        }
    }
    pub fn _with_argon2(salt_path: &std::path::Path) -> Self {
        Self {
            password_hash_function: PasswordHashFunctionKind::_Argon2(salt_path.to_owned()),
        }
    }

    pub fn with_blake2(salt_path: &std::path::Path) -> Self {
        Self {
            password_hash_function: PasswordHashFunctionKind::Blake2(salt_path.to_owned()),
        }
    }

    pub fn build_and_init<R: Runtime>(
        self,
        app_handle: &AppHandle<R>,
        snapshot_path: PathBuf,
        password: String,
    ) -> Result<()> {
        let password_hash_function = self.password_hash_function;

        app_handle.manage(StrongholdCollection::default());
        app_handle.manage(PasswordHashFunction(match password_hash_function {
            PasswordHashFunctionKind::_Argon2(path) => {
                Box::new(move |p| KeyDerivation::argon2(p, &path))
            }
            PasswordHashFunctionKind::Blake2(path) => {
                Box::new(move |p| KeyDerivation::blake2(p, &path))
            }
            PasswordHashFunctionKind::_Custom(f) => f,
        }));

        let collection_state = app_handle.state::<StrongholdCollection>();
        let hash_function_state = app_handle.state::<PasswordHashFunction>();

        initialize(
            collection_state,
            hash_function_state,
            snapshot_path,
            password,
        )?;

        Ok(())
    }
}
