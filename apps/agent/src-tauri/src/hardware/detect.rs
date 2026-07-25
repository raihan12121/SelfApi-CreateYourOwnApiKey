use std::cmp::Ordering;

use chrono::Utc;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

use super::nvidia::detect_nvidia_gpus;
use super::platform::detect_platform_gpus;
use super::types::{
    format_vram_gb, is_software_adapter, recommended_for_inference, GpuDevice, HardwareProfile,
};

pub fn detect_hardware() -> HardwareProfile {
    let mut system = System::new_with_specifics(
        RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything()),
    );
    system.refresh_cpu_all();
    system.refresh_memory();

    let cpu_model = system
        .cpus()
        .first()
        .map(|cpu| cpu.brand().trim().to_string())
        .filter(|value| !value.is_empty());

    let total_ram_bytes = system.total_memory();
    let total_ram_gb = super::types::bytes_to_gb(total_ram_bytes);

    let mut gpus = detect_nvidia_gpus();
    let existing_names: Vec<String> = gpus.iter().map(|gpu| gpu.name.clone()).collect();
    gpus.extend(detect_platform_gpus(&existing_names));
    gpus.sort_by(compare_gpu_priority);
    gpus.dedup_by(|left, right| left.name.eq_ignore_ascii_case(&right.name));

    let primary_gpu = gpus.first().cloned();
    let capability_summary = build_capability_summary(primary_gpu.as_ref(), total_ram_gb);

    HardwareProfile {
        os: std::env::consts::OS.to_string(),
        cpu_model,
        total_ram_bytes,
        total_ram_gb,
        gpus,
        primary_gpu,
        capability_summary,
        detected_at: Utc::now().to_rfc3339(),
    }
}

fn compare_gpu_priority(left: &GpuDevice, right: &GpuDevice) -> Ordering {
    recommended_for_inference(left.vram_bytes)
        .cmp(&recommended_for_inference(right.vram_bytes))
        .reverse()
        .then_with(|| left.vram_bytes.cmp(&right.vram_bytes).reverse())
        .then_with(|| left.name.cmp(&right.name))
}

fn build_capability_summary(primary_gpu: Option<&GpuDevice>, total_ram_gb: f32) -> String {
    let Some(gpu) = primary_gpu else {
        return format!(
            "No discrete GPU detected. SelfAPI can still run smaller models on CPU using {total_ram_gb:.0}GB system RAM, but performance will be limited."
        );
    };

    if is_software_adapter(&gpu.name) {
        return "No compatible GPU detected yet. Connect a discrete GPU with at least 8GB VRAM for the best experience.".to_string();
    }

    let vram_label = gpu
        .vram_gb
        .map(format_vram_gb)
        .unwrap_or_else(|| "unknown VRAM".to_string());

    let model_hint = gpu
        .vram_gb
        .map(model_size_hint)
        .unwrap_or_else(|| "start with smaller quantized models".to_string());

    if gpu.recommended_for_inference {
        format!(
            "You have a {}, {} VRAM — {}.",
            gpu.name, vram_label, model_hint
        )
    } else {
        format!(
            "You have a {}, {} VRAM — usable for experimentation, but 8GB+ VRAM is recommended for larger models.",
            gpu.name, vram_label
        )
    }
}

fn model_size_hint(vram_gb: f32) -> String {
    if vram_gb >= 24.0 {
        "you can run models up to ~30B parameters at Q4 quantization".to_string()
    } else if vram_gb >= 16.0 {
        "you can run models up to ~14B parameters at Q4 quantization".to_string()
    } else if vram_gb >= 12.0 {
        "you can run models up to ~8B parameters at Q4 quantization".to_string()
    } else if vram_gb >= 8.0 {
        "you can run models up to ~7B parameters at Q4 quantization".to_string()
    } else {
        "stick to smaller models (3B or below) at aggressive quantization".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_hardware_profile_shape() {
        let profile = detect_hardware();
        assert!(!profile.os.is_empty());
        assert!(profile.total_ram_bytes > 0);
        assert!(!profile.detected_at.is_empty());
    }
}
