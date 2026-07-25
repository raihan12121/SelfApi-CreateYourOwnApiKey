export type RequestRecord = {
  id: string;
  timestamp: string;
  model: string;
  inputTokens: number;
  outputTokens: number;
  latencyMs: number;
  status: "200 OK" | "401 Unauthorized" | "500 Error";
  servedVia: "local" | "fallback";
  apiKey: string;
};

export type MachineStatus = {
  name: string;
  gpu: string;
  vram: string;
  status: "online" | "offline" | "degraded";
  activeModel: string;
  lastSeen: string;
};

export type ApiKeyItem = {
  id: string;
  keyPrefix: string;
  name: string;
  created: string;
  requestsCount: number;
  status: "active" | "revoked";
};

export type AgentHealthData = {
  status: string;
  agent: string;
  active_model: string;
  requests_handled: number;
  gpu_name: string;
  vram_gb: number;
  vram_used_gb: number;
  public_tunnel_url: string;
  relay_region: string;
  uptime_percentage: number;
  p95_latency_ms: number;
  tier: string;
  price_per_1m_tokens_usd: number;
  total_earnings_usd: number;
  pending_payout_usd: number;
};

const defaultRequests: RequestRecord[] = [
  {
    id: "req_9f821a",
    timestamp: "2 mins ago",
    model: "llama-3.2-3b-instruct",
    inputTokens: 32,
    outputTokens: 145,
    latencyMs: 124,
    status: "200 OK",
    servedVia: "local",
    apiKey: "sk-selfapi-default",
  },
  {
    id: "req_4e112c",
    timestamp: "18 mins ago",
    model: "qwen2.5-7b-instruct",
    inputTokens: 128,
    outputTokens: 312,
    latencyMs: 380,
    status: "200 OK",
    servedVia: "local",
    apiKey: "sk-selfapi-default",
  },
  {
    id: "req_1d908e",
    timestamp: "1 hour ago",
    model: "llama-3.2-3b-instruct",
    inputTokens: 64,
    outputTokens: 92,
    latencyMs: 98,
    status: "200 OK",
    servedVia: "local",
    apiKey: "sk-selfapi-default",
  },
];

export async function checkAgentHealth(): Promise<{ online: boolean; agentData?: AgentHealthData }> {
  try {
    const res = await fetch("http://127.0.0.1:8787/v1/health", { cache: "no-store" });
    if (res.ok) {
      const data = await res.json();
      return { online: true, agentData: data };
    }
  } catch {
    // server offline or unreachable
  }
  return { online: false };
}

export function getStoredRequests(): RequestRecord[] {
  if (typeof window === "undefined") return defaultRequests;
  const raw = localStorage.getItem("selfapi_request_logs");
  if (!raw) {
    localStorage.setItem("selfapi_request_logs", JSON.stringify(defaultRequests));
    return defaultRequests;
  }
  try {
    return JSON.parse(raw);
  } catch {
    return defaultRequests;
  }
}

export function addRequestLog(record: RequestRecord): RequestRecord[] {
  const current = getStoredRequests();
  const updated = [record, ...current];
  if (typeof window !== "undefined") {
    localStorage.setItem("selfapi_request_logs", JSON.stringify(updated));
  }
  return updated;
}
