#[cfg(windows)]
mod inner {
    use serde::Deserialize;
    use wmi::{COMLibrary, WMIConnection};

    use super::super::types::{
        bytes_to_gb, infer_vendor, is_software_adapter, recommended_for_inference, GpuDevice,
    };

    #[derive(Debug, Deserialize)]
    struct Win32VideoController {
        #[serde(rename = "Name")]
        name: Option<String>,
        #[serde(rename = "AdapterRAM")]
        adapter_ram: Option<u32>,
        #[serde(rename = "DriverVersion")]
        driver_version: Option<String>,
        #[serde(rename = "PNPDeviceID")]
        pnp_device_id: Option<String>,
    }

    pub fn detect_platform_gpus(existing_names: &[String]) -> Vec<GpuDevice> {
        let Ok(com) = COMLibrary::new() else {
            return Vec::new();
        };
        let Ok(wmi) = WMIConnection::new(com) else {
            return Vec::new();
        };

        let query = "SELECT Name, AdapterRAM, DriverVersion, PNPDeviceID FROM Win32_VideoController";
        let Ok(controllers): Result<Vec<Win32VideoController>, _> = wmi.raw_query(query) else {
            return Vec::new();
        };

        let mut devices = Vec::new();

        for (index, controller) in controllers.into_iter().enumerate() {
            let Some(name) = controller.name.filter(|value| !value.trim().is_empty()) else {
                continue;
            };

            if is_software_adapter(&name) {
                continue;
            }

            if existing_names
                .iter()
                .any(|existing| names_match(existing, &name))
            {
                continue;
            }

            let vendor = infer_vendor(&name);
            let vram_bytes = controller.adapter_ram.map(|value| value as u64).filter(|bytes| {
                // WMI often reports capped values; keep only when plausible.
                *bytes >= 512 * 1024 * 1024
            });
            let vram_gb = vram_bytes.map(bytes_to_gb);

            devices.push(GpuDevice {
                id: controller
                    .pnp_device_id
                    .unwrap_or_else(|| format!("wmi-{index}"))
                    .replace('\\', "_"),
                vendor: vendor.as_str().to_string(),
                name,
                vram_bytes,
                vram_gb,
                driver_version: controller.driver_version,
                cuda_version: None,
                is_discrete: !matches!(vendor, super::super::types::GpuVendor::Intel),
                recommended_for_inference: recommended_for_inference(vram_bytes),
            });
        }

        devices
    }

    fn names_match(left: &str, right: &str) -> bool {
        normalize_name(left) == normalize_name(right)
    }

    fn normalize_name(name: &str) -> String {
        name.to_lowercase()
            .replace("nvidia ", "")
            .replace("geforce ", "")
            .trim()
            .to_string()
    }
}

#[cfg(windows)]
pub use inner::detect_platform_gpus;

#[cfg(not(windows))]
pub fn detect_platform_gpus(_existing_names: &[String]) -> Vec<super::types::GpuDevice> {
    let mut devices = Vec::new();
    #[cfg(target_os = "macos")]
    {
        use super::types::{GpuDevice, recommended_for_inference, bytes_to_gb};
        let sys = sysinfo::System::new_all();
        let total_mem = sys.total_memory();
        if total_mem > 0 {
            devices.push(GpuDevice {
                id: "apple-silicon-metal".into(),
                vendor: "apple".into(),
                name: "Apple Silicon Metal (Unified Memory)".into(),
                vram_bytes: Some(total_mem),
                vram_gb: Some(bytes_to_gb(total_mem)),
                driver_version: Some("Metal 3".into()),
                cuda_version: None,
                is_discrete: true,
                recommended_for_inference: recommended_for_inference(Some(total_mem)),
            });
        }
    }
    devices
}
