pub mod catalog;
mod download;
mod recommend;
pub mod scan;
mod storage;
mod types;

pub use catalog::find_catalog_model;
pub use download::{start_download, DownloadState};
pub use recommend::{build_library, mark_default_model};
pub use scan::{add_custom_gguf, scan_system_models, ScanResult};
pub use storage::list_installed_models;
pub use types::{
    DownloadProgress, InstalledModel, ModelLibraryResponse,
};


