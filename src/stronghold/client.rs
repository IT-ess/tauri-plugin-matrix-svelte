use iota_stronghold::{KeyProvider, SnapshotPath};
use std::{ops::Deref, path::PathBuf};

use iota_stronghold::Client;
use tauri::State;
use zeroize::Zeroizing;

use super::error::{Error, Result};
use std::path::Path;

use super::{utils::BytesDto, StrongholdCollection};

pub struct Stronghold {
    inner: iota_stronghold::Stronghold,
    path: SnapshotPath,
    keyprovider: KeyProvider,
}

impl Stronghold {
    pub fn new<P: AsRef<Path>>(path: P, password: Vec<u8>) -> Result<Self> {
        let path = SnapshotPath::from_path(path);
        let stronghold = iota_stronghold::Stronghold::default();
        let keyprovider = KeyProvider::try_from(Zeroizing::new(password))?;
        if path.exists() {
            stronghold.load_snapshot(&keyprovider, &path)?;
        }
        Ok(Self {
            inner: stronghold,
            path,
            keyprovider,
        })
    }

    pub fn save(&self) -> Result<()> {
        self.inner
            .commit_with_keyprovider(&self.path, &self.keyprovider)?;
        Ok(())
    }

    pub fn inner(&self) -> &iota_stronghold::Stronghold {
        &self.inner
    }
}

impl Deref for Stronghold {
    type Target = iota_stronghold::Stronghold;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub async fn destroy(
    collection: State<'_, StrongholdCollection>,
    snapshot_path: PathBuf,
) -> Result<()> {
    let mut collection = collection.0.lock().unwrap();
    if let Some(stronghold) = collection.remove(&snapshot_path) {
        if let Err(e) = stronghold.save() {
            collection.insert(snapshot_path, stronghold);
            return Err(e);
        }
    }
    Ok(())
}

pub async fn save(
    collection: State<'_, StrongholdCollection>,
    snapshot_path: PathBuf,
) -> Result<()> {
    let collection = collection.0.lock().unwrap();
    if let Some(stronghold) = collection.get(&snapshot_path) {
        stronghold.save()?;
    }
    Ok(())
}

pub async fn create_client(
    collection: State<'_, StrongholdCollection>,
    snapshot_path: PathBuf,
    client: BytesDto,
) -> Result<()> {
    let stronghold = get_stronghold(collection, snapshot_path)?;
    stronghold.create_client(client)?;
    Ok(())
}

pub async fn load_client(
    collection: State<'_, StrongholdCollection>,
    snapshot_path: PathBuf,
    client: BytesDto,
) -> Result<()> {
    let stronghold = get_stronghold(collection, snapshot_path)?;
    stronghold.load_client(client)?;
    Ok(())
}

pub async fn load_stronghold_client_or_create_it(
    collection: State<'_, StrongholdCollection>,
    snapshot_path: PathBuf,
    client_id: BytesDto,
) -> Result<()> {
    match load_client(collection.clone(), snapshot_path.clone(), client_id.clone()).await {
        Ok(()) => return Ok(()),
        Err(_) => create_client(collection, snapshot_path, client_id).await,
    }
}

fn get_stronghold(
    collection: State<'_, StrongholdCollection>,
    snapshot_path: PathBuf,
) -> Result<iota_stronghold::Stronghold> {
    let collection = collection.0.lock().unwrap();
    if let Some(stronghold) = collection.get(&snapshot_path) {
        Ok(stronghold.inner().clone())
    } else {
        Err(Error::StrongholdNotInitialized)
    }
}

pub fn get_client(
    collection: State<'_, StrongholdCollection>,
    snapshot_path: PathBuf,
    client: BytesDto,
) -> Result<Client> {
    let collection = collection.0.lock().unwrap();
    if let Some(stronghold) = collection.get(&snapshot_path) {
        stronghold.get_client(client).map_err(Into::into)
    } else {
        Err(Error::StrongholdNotInitialized)
    }
}
