use crate::hardware::HardwareProfile;

use super::catalog::{curated_models, find_quantization};
use super::storage::installed_model_ids;
use super::types::{ModelFit, ModelLibraryResponse, ModelRecommendation, QuantizationOption};

const HEADROOM_FACTOR: f32 = 0.85;

pub fn build_library(profile: &HardwareProfile) -> ModelLibraryResponse {
    let (available_vram_gb, memory_source) = inference_budget(profile);
    let installed_ids = installed_model_ids();

    let mut models: Vec<ModelRecommendation> = curated_models()
        .into_iter()
        .filter_map(|model| recommend_model(&model, available_vram_gb, &installed_ids))
        .collect();

    models.sort_by(|left, right| {
        fit_rank(&left.fit)
            .cmp(&fit_rank(&right.fit))
            .then(left.is_default.cmp(&right.is_default).reverse())
            .then(
                left.model
                    .parameter_count_b
                    .partial_cmp(&right.model.parameter_count_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .reverse(),
            )
            .then(left.model.name.cmp(&right.model.name))
    });

    let default_model_id = models
        .iter()
        .find(|model| model.is_default)
        .or_else(|| models.iter().find(|model| matches!(model.fit, ModelFit::Recommended)))
        .map(|model| model.model.id.clone());

    ModelLibraryResponse {
        available_vram_gb,
        memory_source,
        default_model_id,
        models,
    }
}

fn recommend_model(
    model: &super::types::CatalogModel,
    available_vram_gb: f32,
    installed_ids: &std::collections::HashSet<String>,
) -> Option<ModelRecommendation> {
    let available_quantizations: Vec<QuantizationOption> = model
        .quantizations
        .iter()
        .filter(|quant| quant.min_vram_gb <= available_vram_gb)
        .cloned()
        .collect();

    let best_quant = pick_best_quantization(model, available_vram_gb)?;
    let fit = classify_fit(best_quant.min_vram_gb, available_vram_gb);

    if matches!(fit, ModelFit::TooLarge) {
        return Some(ModelRecommendation {
            model: model.clone(),
            recommended_quantization: best_quant.clone(),
            available_quantizations: model.quantizations.clone(),
            fit,
            is_default: false,
            installed: installed_ids.contains(&model.id),
        });
    }

    Some(ModelRecommendation {
        model: model.clone(),
        recommended_quantization: best_quant.clone(),
        available_quantizations,
        fit,
        is_default: false,
        installed: installed_ids.contains(&model.id),
    })
}

fn pick_best_quantization(
    model: &super::types::CatalogModel,
    available_vram_gb: f32,
) -> Option<QuantizationOption> {
    let mut candidates: Vec<&QuantizationOption> = model
        .quantizations
        .iter()
        .filter(|quant| quant.min_vram_gb <= available_vram_gb)
        .collect();

    if candidates.is_empty() {
        return model.quantizations.first().cloned();
    }

    candidates.sort_by(|left, right| {
        left.min_vram_gb
            .partial_cmp(&right.min_vram_gb)
            .unwrap_or(std::cmp::Ordering::Equal)
            .reverse()
    });

    candidates.first().map(|quant| (*quant).clone())
}

fn classify_fit(required_vram_gb: f32, available_vram_gb: f32) -> ModelFit {
    if required_vram_gb > available_vram_gb {
        ModelFit::TooLarge
    } else if required_vram_gb <= available_vram_gb * HEADROOM_FACTOR {
        ModelFit::Recommended
    } else {
        ModelFit::Caution
    }
}

fn inference_budget(profile: &HardwareProfile) -> (f32, String) {
    if let Some(gpu) = &profile.primary_gpu {
        if let Some(vram_gb) = gpu.vram_gb {
            return (vram_gb, format!("{} VRAM", gpu.name));
        }
    }

    let ram_budget = (profile.total_ram_gb * 0.45).max(3.0);
    (ram_budget, "system RAM (CPU fallback)".to_string())
}

fn fit_rank(fit: &ModelFit) -> u8 {
    match fit {
        ModelFit::Recommended => 0,
        ModelFit::Caution => 1,
        ModelFit::TooLarge => 2,
    }
}

const DEFAULT_HEADROOM_FACTOR: f32 = 0.65;

pub fn mark_default_model(library: &mut ModelLibraryResponse) {
    let budget = library.available_vram_gb * DEFAULT_HEADROOM_FACTOR;

    let best = library
        .models
        .iter()
        .filter(|model| matches!(model.fit, ModelFit::Recommended))
        .filter(|model| {
            model
                .available_quantizations
                .iter()
                .any(|quant| quant.min_vram_gb <= budget)
        })
        .max_by(|left, right| {
            left.model
                .parameter_count_b
                .partial_cmp(&right.model.parameter_count_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .or_else(|| {
            library
                .models
                .iter()
                .find(|model| matches!(model.fit, ModelFit::Recommended))
        });

    if let Some(best) = best {
        let best_id = best.model.id.clone();
        library.default_model_id = Some(best_id.clone());
        for model in &mut library.models {
            model.is_default = model.model.id == best_id;
            if model.is_default {
                if let Some(quant) = model
                    .available_quantizations
                    .iter()
                    .filter(|quant| quant.min_vram_gb <= budget)
                    .max_by(|left, right| {
                        left.min_vram_gb
                            .partial_cmp(&right.min_vram_gb)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                {
                    model.recommended_quantization = quant.clone();
                }
            }
        }
    }
}

pub fn resolve_selection(
    model_id: &str,
    quantization_id: &str,
    profile: &HardwareProfile,
) -> Result<(super::types::CatalogModel, QuantizationOption), String> {
    let library = {
        let mut library = build_library(profile);
        mark_default_model(&mut library);
        library
    };

    let recommendation = library
        .models
        .into_iter()
        .find(|model| model.model.id == model_id)
        .ok_or_else(|| format!("Unknown model: {model_id}"))?;

    if matches!(recommendation.fit, ModelFit::TooLarge) {
        return Err(format!(
            "{} requires more VRAM than your {} can provide.",
            recommendation.model.name, library.memory_source
        ));
    }

    let quantization = find_quantization(&recommendation.model, quantization_id)
        .cloned()
        .unwrap_or(recommendation.recommended_quantization.clone());

    if quantization.min_vram_gb > library.available_vram_gb {
        return Err(format!(
            "{} {} needs at least {:.1}GB VRAM.",
            recommendation.model.name, quantization.label, quantization.min_vram_gb
        ));
    }

    Ok((recommendation.model, quantization))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::hardware::GpuDevice;

    fn profile_with_vram(vram_gb: f32) -> HardwareProfile {
        let gpu = GpuDevice {
            id: "test-gpu".into(),
            vendor: "nvidia".into(),
            name: "Test GPU".into(),
            vram_bytes: Some((vram_gb * 1024.0 * 1024.0 * 1024.0) as u64),
            vram_gb: Some(vram_gb),
            driver_version: Some("999".into()),
            cuda_version: Some("12.0".into()),
            is_discrete: true,
            recommended_for_inference: vram_gb >= 8.0,
        };

        HardwareProfile {
            os: "windows".into(),
            cpu_model: Some("Test CPU".into()),
            total_ram_bytes: 16 * 1024 * 1024 * 1024,
            total_ram_gb: 16.0,
            primary_gpu: Some(gpu.clone()),
            gpus: vec![gpu],
            capability_summary: String::new(),
            detected_at: Utc::now().to_rfc3339(),
        }
    }

    #[test]
    fn recommends_7b_model_for_12gb_vram() {
        let mut library = build_library(&profile_with_vram(12.0));
        mark_default_model(&mut library);

        let default = library
            .models
            .iter()
            .find(|model| model.is_default)
            .expect("default model");

        assert!(default.model.parameter_count_b >= 7.0);
        assert!(default.model.parameter_count_b <= 8.0);
        assert!(matches!(default.fit, ModelFit::Recommended));
    }

    #[test]
    fn recommends_14b_model_for_16gb_vram() {
        let mut library = build_library(&profile_with_vram(16.0));
        mark_default_model(&mut library);

        let default = library
            .models
            .iter()
            .find(|model| model.is_default)
            .expect("default model");

        assert!(default.model.parameter_count_b >= 14.0);
        assert!(matches!(default.fit, ModelFit::Recommended));
    }

    #[test]
    fn recommends_small_model_for_8gb_vram() {
        let mut library = build_library(&profile_with_vram(8.0));
        mark_default_model(&mut library);

        let default = library
            .models
            .iter()
            .find(|model| model.is_default)
            .expect("default model");

        assert!(default.model.parameter_count_b <= 7.0);
        assert!(matches!(default.fit, ModelFit::Recommended));
    }
}
