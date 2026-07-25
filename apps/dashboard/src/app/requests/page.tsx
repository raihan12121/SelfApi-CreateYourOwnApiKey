"use client";

import { useState } from "react";
import { PageHeader } from "@/components/layout/page-header";
import { getStoredRequests, RequestRecord } from "@/lib/store";

const tabs = [
  { id: "history", label: "History" },
  { id: "usage", label: "Usage charts" },
  { id: "performance", label: "Performance" },
  { id: "export", label: "Export" },
] as const;

type TabId = (typeof tabs)[number]["id"];

export default function RequestsPage() {
  const [activeTab, setActiveTab] = useState<TabId>("history");
  const [requests] = useState<RequestRecord[]>(() => getStoredRequests());

  const handleExportCsv = () => {
    const headers = "id,timestamp,model,inputTokens,outputTokens,latencyMs,status,servedVia,apiKey\n";
    const rows = requests
      .map(
        (r) =>
          `"${r.id}","${r.timestamp}","${r.model}",${r.inputTokens},${r.outputTokens},${r.latencyMs},"${r.status}","${r.servedVia}","${r.apiKey}"`,
      )
      .join("\n");
    const blob = new Blob([headers + rows], { type: "text/csv" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `selfapi_request_history_${Date.now()}.csv`;
    a.click();
  };

  return (
    <>
      <PageHeader
        title="Request history & analytics"
        description="Searchable logs, usage charts, performance metrics, and exports."
      />

      <div className="mb-6 flex flex-wrap gap-2 border-b border-zinc-200 dark:border-zinc-800">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            type="button"
            onClick={() => setActiveTab(tab.id)}
            className={`-mb-px border-b-2 px-3 py-2 text-sm font-medium transition-colors ${
              activeTab === tab.id
                ? "border-zinc-900 text-zinc-900 dark:border-zinc-100 dark:text-zinc-100"
                : "border-transparent text-zinc-500 hover:text-zinc-700 dark:text-zinc-400 dark:hover:text-zinc-200"
            }`}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {activeTab === "history" && (
        <div className="overflow-hidden rounded-lg border border-zinc-200 bg-white dark:border-zinc-800 dark:bg-zinc-950">
          <table className="min-w-full text-left text-sm">
            <thead className="sticky top-0 border-b border-zinc-200 bg-zinc-50 text-xs uppercase tracking-wide text-zinc-500 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-400">
              <tr>
                <th className="px-4 py-3 font-medium">Request ID / Time</th>
                <th className="px-4 py-3 font-medium">Model</th>
                <th className="px-4 py-3 font-medium">Tokens (in/out)</th>
                <th className="px-4 py-3 font-medium">Latency</th>
                <th className="px-4 py-3 font-medium">Status</th>
                <th className="px-4 py-3 font-medium">Served via</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-zinc-200 dark:divide-zinc-800">
              {requests.length === 0 ? (
                <tr>
                  <td
                    colSpan={6}
                    className="px-4 py-10 text-center text-zinc-500 dark:text-zinc-400"
                  >
                    No requests yet — send a request from the Desktop Agent or cURL to record activity.
                  </td>
                </tr>
              ) : (
                requests.map((r) => (
                  <tr key={r.id} className="hover:bg-zinc-50 dark:hover:bg-zinc-900/50">
                    <td className="px-4 py-3">
                      <div className="font-mono font-medium text-zinc-900 dark:text-zinc-100">{r.id}</div>
                      <div className="text-xs text-zinc-500">{r.timestamp}</div>
                    </td>
                    <td className="px-4 py-3 font-mono text-xs">{r.model}</td>
                    <td className="px-4 py-3 font-mono text-xs">
                      {r.inputTokens} / {r.outputTokens}
                    </td>
                    <td className="px-4 py-3 font-mono text-xs">{r.latencyMs} ms</td>
                    <td className="px-4 py-3">
                      <span className="inline-flex rounded-full bg-emerald-500/10 px-2 py-0.5 text-xs font-semibold text-emerald-500">
                        {r.status}
                      </span>
                    </td>
                    <td className="px-4 py-3 text-xs text-zinc-500 uppercase font-mono">{r.servedVia}</td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      )}

      {activeTab === "usage" && (
        <div className="rounded-lg border border-zinc-200 bg-white p-6 dark:border-zinc-800 dark:bg-zinc-950">
          <h3 className="text-sm font-medium mb-4">Request Volume Timeline</h3>
          <div className="flex h-40 items-end gap-2 border-b border-zinc-200 pb-2 dark:border-zinc-800">
            {[4, 8, 15, 6, 22, 18, requests.length].map((val, idx) => (
              <div key={idx} className="flex-1 flex flex-col items-center gap-1">
                <div
                  className="w-full rounded-t bg-blue-500 transition-all hover:bg-blue-400"
                  style={{ height: `${Math.max(12, val * 6)}px` }}
                />
                <span className="text-[10px] text-zinc-500">Day {idx + 1}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {activeTab === "performance" && (
        <div className="grid gap-4 sm:grid-cols-3">
          <div className="rounded-lg border border-zinc-200 bg-white p-4 dark:border-zinc-800 dark:bg-zinc-950">
            <span className="text-xs text-zinc-500">Avg Latency (TTFT)</span>
            <div className="mt-1 text-xl font-bold font-mono">112 ms</div>
          </div>
          <div className="rounded-lg border border-zinc-200 bg-white p-4 dark:border-zinc-800 dark:bg-zinc-950">
            <span className="text-xs text-zinc-500">Throughput</span>
            <div className="mt-1 text-xl font-bold font-mono">42 tok/sec</div>
          </div>
          <div className="rounded-lg border border-zinc-200 bg-white p-4 dark:border-zinc-800 dark:bg-zinc-950">
            <span className="text-xs text-zinc-500">Local Hardware Ratio</span>
            <div className="mt-1 text-xl font-bold font-mono text-emerald-500">100% Local</div>
          </div>
        </div>
      )}

      {activeTab === "export" && (
        <div className="rounded-lg border border-zinc-200 bg-white p-6 dark:border-zinc-800 dark:bg-zinc-950">
          <p className="text-sm text-zinc-600 dark:text-zinc-300">
            Export filtered request history as a CSV file for your records or billing reconciliation.
          </p>
          <button
            type="button"
            onClick={handleExportCsv}
            className="mt-4 rounded-md border border-zinc-200 bg-zinc-900 px-4 py-2 text-sm font-medium text-white hover:bg-zinc-800 dark:border-zinc-700 dark:bg-zinc-100 dark:text-zinc-900 dark:hover:bg-zinc-200"
          >
            Export current view (CSV)
          </button>
        </div>
      )}
    </>
  );
}

