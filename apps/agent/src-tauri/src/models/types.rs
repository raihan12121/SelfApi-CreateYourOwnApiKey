use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelFit {
    Recommended,
    Caution,
    TooLarge,
}

#[derive(Debug, Clone, Serialize)]
pub struct QuantizationOption {
    pub id: String,
    pub label: String,
    pub min_vram_gb: f32,
    pub file_size_bytes: u64,
    pub download_url: String,
    pub filename: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CatalogModel {
    pub id: String,
    pub name: String,
    pub family: String,
    pub parameter_count_b: f32,
    pub description: String,
    pub quantizations: Vec<QuantizationOption>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelRecommendation {
    pub model: CatalogModel,
    pub recommended_quantization: QuantizationOption,
    pub available_quantizations: Vec<QuantizationOption>,
    pub fit: ModelFit,
    pub is_default: bool,
    pub installed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelLibraryResponse {
    pub available_vram_gb: f32,
    pub memory_source: String,
    pub default_model_id: Option<String>,
    pub models: Vec<ModelRecommendation>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadStatus {
    #[allow(dead_code)]
    Idle,
    Downloading,
    Completed,
    Failed,
    #[allow(dead_code)]
    Cancelled,
}

#[derive(Debug, Clone, Serialize)]
pub struct DownloadProgress {
    pub model_id: String,
    pub quantization_id: String,
    pub status: DownloadStatus,
    pub bytes_downloaded: u64,
    pub total_bytes: Option<u64>,
    pub progress_percent: Option<f32>,
    pub file_path: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstalledModel {
    pub model_id: String,
    pub model_name: String,
    pub quantization_id: String,
    pub filename: String,
    pub file_path: String,
    pub file_size_bytes: u64,
    pub installed_at: String,
}

impl ModelFit {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Recommended => "recommended",
            Self::Caution => "caution",
            Self::TooLarge => "too_large",
        }
    }
}
