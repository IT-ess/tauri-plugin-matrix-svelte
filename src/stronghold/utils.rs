use std::path::Path;

use blake2::{Blake2b512, Digest};
use rand_chacha::ChaCha20Rng;
use rand_core::{RngCore, SeedableRng};
use serde::Deserialize;

#[derive(Deserialize, Hash, Eq, PartialEq, Ord, PartialOrd, Clone)]
#[serde(untagged)]
pub enum BytesDto {
    Text(String),
    Raw(Vec<u8>),
}

impl AsRef<[u8]> for BytesDto {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Text(t) => t.as_ref(),
            Self::Raw(b) => b.as_ref(),
        }
    }
}

impl From<BytesDto> for Vec<u8> {
    fn from(v: BytesDto) -> Self {
        match v {
            BytesDto::Text(t) => t.as_bytes().to_vec(),
            BytesDto::Raw(b) => b,
        }
    }
}

const HASH_LENGTH: usize = 32;

pub struct KeyDerivation {}

impl KeyDerivation {
    /// Will create a key from [`password`] and a generated salt.
    /// Salt will be generated to file [`salt_path`] or taken from it
    /// if file already exists
    pub fn argon2(password: &str, salt_path: &Path) -> Vec<u8> {
        let mut salt = [0u8; HASH_LENGTH];
        create_or_get_salt(&mut salt, salt_path);

        argon2::hash_raw(password.as_bytes(), &salt, &Default::default())
            .expect("Failed to generate hash for password")
    }
    pub fn blake2(password: &str, salt_path: &Path) -> Vec<u8> {
        let mut salt = [0u8; HASH_LENGTH];
        create_or_get_salt(&mut salt, salt_path);

        let mut hasher = Blake2b512::new();
        hasher.update(salt);
        hasher.update(password);
        hasher.finalize().to_vec()[..32].to_vec()
    }
}

fn create_or_get_salt(salt: &mut [u8], salt_path: &Path) {
    if salt_path.is_file() {
        // Get existing salt
        let tmp = std::fs::read(salt_path).unwrap();
        salt.clone_from_slice(&tmp);
    } else {
        // Generate new salt
        let mut generator = ChaCha20Rng::from_entropy();
        generator.fill_bytes(salt);
        std::fs::write(salt_path, salt).expect("Failed to write salt for Stronghold")
    }
}
