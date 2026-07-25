use super::types::{CatalogModel, QuantizationOption};

fn quant(
    id: &str,
    label: &str,
    min_vram_gb: f32,
    file_size_bytes: u64,
    download_url: &str,
    filename: &str,
) -> QuantizationOption {
    QuantizationOption {
        id: id.to_string(),
        label: label.to_string(),
        min_vram_gb,
        file_size_bytes,
        download_url: download_url.to_string(),
        filename: filename.to_string(),
    }
}

pub fn curated_models() -> Vec<CatalogModel> {
    vec![
        CatalogModel {
            id: "llama-3.2-3b-instruct".into(),
            name: "Llama 3.2 3B Instruct".into(),
            family: "Meta Llama".into(),
            parameter_count_b: 3.0,
            description:
                "Fast, lightweight general-purpose model. Great first pick for 8GB GPUs or CPU fallback."
                    .into(),
            quantizations: vec![
                quant(
                    "Q4_K_M",
                    "Q4_K_M (balanced)",
                    3.0,
                    2_010_000_000,
                    "https://huggingface.co/bartowski/Llama-3.2-3B-Instruct-GGUF/resolve/main/Llama-3.2-3B-Instruct-Q4_K_M.gguf",
                    "Llama-3.2-3B-Instruct-Q4_K_M.gguf",
                ),
                quant(
                    "Q5_K_M",
                    "Q5_K_M (higher quality)",
                    3.5,
                    2_350_000_000,
                    "https://huggingface.co/bartowski/Llama-3.2-3B-Instruct-GGUF/resolve/main/Llama-3.2-3B-Instruct-Q5_K_M.gguf",
                    "Llama-3.2-3B-Instruct-Q5_K_M.gguf",
                ),
            ],
        },
        CatalogModel {
            id: "phi-3.5-mini-instruct".into(),
            name: "Phi-3.5 Mini Instruct".into(),
            family: "Microsoft Phi".into(),
            parameter_count_b: 3.8,
            description:
                "Strong reasoning for its size. Ideal when VRAM is tight but you want better quality than 3B."
                    .into(),
            quantizations: vec![
                quant(
                    "Q4_K_M",
                    "Q4_K_M (balanced)",
                    3.2,
                    2_300_000_000,
                    "https://huggingface.co/bartowski/Phi-3.5-mini-instruct-GGUF/resolve/main/Phi-3.5-mini-instruct-Q4_K_M.gguf",
                    "Phi-3.5-mini-instruct-Q4_K_M.gguf",
                ),
                quant(
                    "Q8_0",
                    "Q8_0 (best quality)",
                    5.0,
                    4_100_000_000,
                    "https://huggingface.co/bartowski/Phi-3.5-mini-instruct-GGUF/resolve/main/Phi-3.5-mini-instruct-Q8_0.gguf",
                    "Phi-3.5-mini-instruct-Q8_0.gguf",
                ),
            ],
        },
        CatalogModel {
            id: "qwen2.5-7b-instruct".into(),
            name: "Qwen2.5 7B Instruct".into(),
            family: "Alibaba Qwen".into(),
            parameter_count_b: 7.0,
            description:
                "Excellent 7B all-rounder for coding and chat. Recommended default for 12GB GPUs."
                    .into(),
            quantizations: vec![
                quant(
                    "Q4_K_M",
                    "Q4_K_M (balanced)",
                    5.0,
                    4_680_000_000,
                    "https://huggingface.co/bartowski/Qwen2.5-7B-Instruct-GGUF/resolve/main/Qwen2.5-7B-Instruct-Q4_K_M.gguf",
                    "Qwen2.5-7B-Instruct-Q4_K_M.gguf",
                ),
                quant(
                    "Q5_K_M",
                    "Q5_K_M (higher quality)",
                    6.0,
                    5_450_000_000,
                    "https://huggingface.co/bartowski/Qwen2.5-7B-Instruct-GGUF/resolve/main/Qwen2.5-7B-Instruct-Q5_K_M.gguf",
                    "Qwen2.5-7B-Instruct-Q5_K_M.gguf",
                ),
            ],
        },
        CatalogModel {
            id: "mistral-7b-instruct-v0.3".into(),
            name: "Mistral 7B Instruct v0.3".into(),
            family: "Mistral AI".into(),
            parameter_count_b: 7.0,
            description:
                "Proven 7B workhorse with fast inference. Strong choice for general API workloads."
                    .into(),
            quantizations: vec![
                quant(
                    "Q4_K_M",
                    "Q4_K_M (balanced)",
                    5.0,
                    4_370_000_000,
                    "https://huggingface.co/bartowski/Mistral-7B-Instruct-v0.3-GGUF/resolve/main/Mistral-7B-Instruct-v0.3-Q4_K_M.gguf",
                    "Mistral-7B-Instruct-v0.3-Q4_K_M.gguf",
                ),
                quant(
                    "Q5_K_M",
                    "Q5_K_M (higher quality)",
                    6.0,
                    5_150_000_000,
                    "https://huggingface.co/bartowski/Mistral-7B-Instruct-v0.3-GGUF/resolve/main/Mistral-7B-Instruct-v0.3-Q5_K_M.gguf",
                    "Mistral-7B-Instruct-v0.3-Q5_K_M.gguf",
                ),
            ],
        },
        CatalogModel {
            id: "llama-3.1-8b-instruct".into(),
            name: "Llama 3.1 8B Instruct".into(),
            family: "Meta Llama".into(),
            parameter_count_b: 8.0,
            description:
                "High-quality 8B model for production-style chat APIs when you have 12GB+ VRAM."
                    .into(),
            quantizations: vec![
                quant(
                    "Q4_K_M",
                    "Q4_K_M (balanced)",
                    5.5,
                    4_920_000_000,
                    "https://huggingface.co/bartowski/Meta-Llama-3.1-8B-Instruct-GGUF/resolve/main/Meta-Llama-3.1-8B-Instruct-Q4_K_M.gguf",
                    "Meta-Llama-3.1-8B-Instruct-Q4_K_M.gguf",
                ),
                quant(
                    "Q5_K_M",
                    "Q5_K_M (higher quality)",
                    6.5,
                    5_730_000_000,
                    "https://huggingface.co/bartowski/Meta-Llama-3.1-8B-Instruct-GGUF/resolve/main/Meta-Llama-3.1-8B-Instruct-Q5_K_M.gguf",
                    "Meta-Llama-3.1-8B-Instruct-Q5_K_M.gguf",
                ),
            ],
        },
        CatalogModel {
            id: "qwen2.5-14b-instruct".into(),
            name: "Qwen2.5 14B Instruct".into(),
            family: "Alibaba Qwen".into(),
            parameter_count_b: 14.0,
            description:
                "Larger model for 16GB+ GPUs. Better quality for complex prompts and longer context."
                    .into(),
            quantizations: vec![
                quant(
                    "Q4_K_M",
                    "Q4_K_M (balanced)",
                    10.0,
                    8_900_000_000,
                    "https://huggingface.co/bartowski/Qwen2.5-14B-Instruct-GGUF/resolve/main/Qwen2.5-14B-Instruct-Q4_K_M.gguf",
                    "Qwen2.5-14B-Instruct-Q4_K_M.gguf",
                ),
                quant(
                    "Q5_K_M",
                    "Q5_K_M (higher quality)",
                    12.0,
                    10_200_000_000,
                    "https://huggingface.co/bartowski/Qwen2.5-14B-Instruct-GGUF/resolve/main/Qwen2.5-14B-Instruct-Q5_K_M.gguf",
                    "Qwen2.5-14B-Instruct-Q5_K_M.gguf",
                ),
            ],
        },
    ]
}

pub fn find_catalog_model(model_id: &str) -> Option<CatalogModel> {
    curated_models()
        .into_iter()
        .find(|model| model.id == model_id)
}

pub fn find_quantization<'a>(
    model: &'a CatalogModel,
    quantization_id: &str,
) -> Option<&'a QuantizationOption> {
    model
        .quantizations
        .iter()
        .find(|quant| quant.id == quantization_id)
}
