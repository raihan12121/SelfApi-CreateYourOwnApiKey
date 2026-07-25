export type GpuDevice = {
  id: string;
  vendor: string;
  name: string;
  vram_bytes: number | null;
  vram_gb: number | null;
  driver_version: string | null;
  cuda_version: string | null;
  is_discrete: boolean;
  recommended_for_inference: boolean;
};

export type HardwareProfile = {
  os: string;
  cpu_model: string | null;
  total_ram_bytes: number;
  total_ram_gb: number;
  gpus: GpuDevice[];
  primary_gpu: GpuDevice | null;
  capability_summary: string;
  detected_at: string;
};

export type QuantizationOption = {
  id: string;
  label: string;
  min_vram_gb: number;
  file_size_bytes: number;
  download_url: string;
  filename: string;
};

export type CatalogModel = {
  id: string;
  name: string;
  family: string;
  parameter_count_b: number;
  description: string;
  quantizations: QuantizationOption[];
};

export type ModelFit = "recommended" | "caution" | "too_large";

export type ModelRecommendation = {
  model: CatalogModel;
  recommended_quantization: QuantizationOption;
  available_quantizations: QuantizationOption[];
  fit: ModelFit;
  is_default: boolean;
  installed: boolean;
};

export type ModelLibraryResponse = {
  available_vram_gb: number;
  memory_source: string;
  default_model_id: string | null;
  models: ModelRecommendation[];
};

export type DownloadStatus =
  | "idle"
  | "downloading"
  | "completed"
  | "failed"
  | "cancelled";

export type DownloadProgress = {
  model_id: string;
  quantization_id: string;
  status: DownloadStatus;
  bytes_downloaded: number;
  total_bytes: number | null;
  progress_percent: number | null;
  file_path: string | null;
  error: string | null;
};

export type InstalledModel = {
  model_id: string;
  model_name: string;
  quantization_id: string;
  filename: string;
  file_path: string;
  file_size_bytes: number;
  installed_at: string;
};

export type CodeSnippet = {
  language: string;
  label: string;
  code: string;
};

export type ApiAccessResponse = {
  key_id: string;
  secret_key: string;
  endpoint_url: string;
  public_endpoint_url: string | null;
  model_id: string;
  model_name: string;
  created_at: string;
  snippets: CodeSnippet[];
};

export type LocalServerStatus = {
  running: boolean;
  port: number;
  endpoint_url: string;
  active_model: string | null;
  requests_handled: number;
};

export type ActiveModelRuntimeInfo = {
  model_id: string;
  model_name: string;
  quantization_id: string;
  file_path: string;
  offload_gpu_layers: number;
  runner_type: string;
  status: string;
};


