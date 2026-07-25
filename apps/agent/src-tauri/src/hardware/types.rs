use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Apple,
    Unknown,
}

impl GpuVendor {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Nvidia => "nvidia",
            Self::Amd => "amd",
            Self::Intel => "intel",
            Self::Apple => "apple",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GpuDevice {
    pub id: String,
    pub vendor: String,
    pub name: String,
    pub vram_bytes: Option<u64>,
    pub vram_gb: Option<f32>,
    pub driver_version: Option<String>,
    pub cuda_version: Option<String>,
    pub is_discrete: bool,
    pub recommended_for_inference: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct HardwareProfile {
    pub os: String,
    pub cpu_model: Option<String>,
    pub total_ram_bytes: u64,
    pub total_ram_gb: f32,
    pub gpus: Vec<GpuDevice>,
    pub primary_gpu: Option<GpuDevice>,
    pub capability_summary: String,
    pub detected_at: String,
}

pub fn infer_vendor(name: &str) -> GpuVendor {
    let lower = name.to_lowercase();

    if lower.contains("nvidia") || lower.contains("geforce") || lower.contains("quadro") {
        GpuVendor::Nvidia
    } else if lower.contains("amd") || lower.contains("radeon") {
        GpuVendor::Amd
    } else if lower.contains("intel") {
        GpuVendor::Intel
    } else if lower.contains("apple") {
        GpuVendor::Apple
    } else {
        GpuVendor::Unknown
    }
}

pub fn is_software_adapter(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("microsoft basic")
        || lower.contains("remote desktop")
        || lower.contains("virtual display")
        || lower.contains("meta virtual")
}

pub fn recommended_for_inference(vram_bytes: Option<u64>) -> bool {
    vram_bytes.is_some_and(|bytes| bytes >= 8 * 1024 * 1024 * 1024)
}

pub fn bytes_to_gb(bytes: u64) -> f32 {
    (bytes as f64 / (1024.0 * 1024.0 * 1024.0)) as f32
}

pub fn format_vram_gb(vram_gb: f32) -> String {
    if vram_gb >= 10.0 {
        format!("{vram_gb:.0}GB")
    } else {
        format!("{vram_gb:.1}GB")
    }
}
