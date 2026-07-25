use nvml_wrapper::Nvml;

use super::types::{
    bytes_to_gb, recommended_for_inference, GpuDevice, GpuVendor,
};

pub fn detect_nvidia_gpus() -> Vec<GpuDevice> {
    let Ok(nvml) = Nvml::init() else {
        return Vec::new();
    };

    let Ok(count) = nvml.device_count() else {
        return Vec::new();
    };

    let driver_version = nvml.sys_driver_version().ok();
    let cuda_version = nvml.sys_cuda_driver_version().ok().map(|version| {
        let major = version / 1000;
        let minor = (version % 1000) / 10;
        format!("{major}.{minor}")
    });

    let mut devices = Vec::new();

    for index in 0..count {
        let Ok(device) = nvml.device_by_index(index) else {
            continue;
        };

        let Ok(name) = device.name() else {
            continue;
        };

        let memory = device.memory_info().ok();
        let vram_bytes = memory.map(|info| info.total);
        let vram_gb = vram_bytes.map(bytes_to_gb);

        devices.push(GpuDevice {
            id: format!("nvidia-{index}"),
            vendor: GpuVendor::Nvidia.as_str().to_string(),
            name,
            vram_bytes,
            vram_gb,
            driver_version: driver_version.clone(),
            cuda_version: cuda_version.clone(),
            is_discrete: true,
            recommended_for_inference: recommended_for_inference(vram_bytes),
        });
    }

    devices
}
