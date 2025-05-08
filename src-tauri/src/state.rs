use std::path::PathBuf;

use crate::storage::Storage;

#[derive(Default)]
pub struct AppState {
    pub storage: Storage,
    pub storage_path: PathBuf,
}
