"use client";

import { useState } from "react";
import { PageHeader } from "@/components/layout/page-header";

type AlertRule = {
  id: string;
  name: string;
  condition: string;
  channel: string;
  enabled: boolean;
};

type AlertEvent = {
  id: string;
  time: string;
  type: string;
  severity: "critical" | "warning" | "info";
  message: string;
};

const initialRules: AlertRule[] = [
  {
    id: "rule_1",
    name: "Machine Offline Alert",
    condition: "Primary desktop agent disconnects > 30s",
    channel: "Email & Webhook",
    enabled: true,
  },
  {
    id: "rule_2",
    name: "High VRAM Exhaustion",
    condition: "VRAM utilization > 92%",
    channel: "Webhook (Slack)",
    enabled: true,
  },
  {
    id: "rule_3",
    name: "Cloud Fallback SLA Breach",
    condition: "Local queue latency > 2500ms (Triggered Fallback)",
    channel: "Dashboard & Email",
    enabled: true,
  },
];

const initialEvents: AlertEvent[] = [
  {
    id: "evt_1",
    time: "12 mins ago",
    type: "SLA_LATENCY_BREACH",
    severity: "warning",
    message: "Local queue latency peaked at 2,840ms. Switched request to Groq Cloud Fallback.",
  },
  {
    id: "evt_2",
    time: "1 hour ago",
    type: "VRAM_SPIKE",
    severity: "info",
    message: "VRAM allocation reached 10.4 GB / 12 GB during Qwen 2.5 7B evaluation batch.",
  },
  {
    id: "evt_3",
    time: "Yesterday",
    type: "MACHINE_RECONNECTED",
    severity: "info",
    message: "Desktop Workstation (RTX 4070) re-established tunnel via US-East Relay.",
  },
];

export default function AlertsPage() {
  const [rules, setRules] = useState<AlertRule[]>(initialRules);
  const [events] = useState<AlertEvent[]>(initialEvents);
  const [webhookUrl, setWebhookUrl] = useState("https://hooks.slack.com/services/T00/B00/X00");
  const [savedToast, setSavedToast] = useState(false);

  const toggleRule = (id: string) => {
    setRules((prev) =>
      prev.map((r) => (r.id === id ? { ...r, enabled: !r.enabled } : r)),
    );
  };

  const handleSaveWebhook = (e: React.FormEvent) => {
    e.preventDefault();
    setSavedToast(true);
    setTimeout(() => setSavedToast(false), 2000);
  };

  return (
    <>
      <PageHeader
        title="Alerts & notifications"
        description="Email and webhook alerts for error spikes, offline machines, and SLA fallback triggers."
      />

      <div className="space-y-6">
        {savedToast && (
          <div className="rounded-lg bg-emerald-500/10 border border-emerald-500/30 p-3 text-xs font-medium text-emerald-400">
            ✓ Alert webhook settings saved successfully.
          </div>
        )}

        {/* Active Alert Rules */}
        <section className="rounded-lg border border-zinc-200 bg-white p-5 dark:border-zinc-800 dark:bg-zinc-950">
          <h2 className="text-sm font-semibold text-zinc-900 dark:text-zinc-100 mb-4">Configured Alert Rules</h2>
          <div className="space-y-3">
            {rules.map((rule) => (
              <div
                key={rule.id}
                className="flex items-center justify-between rounded-md border border-zinc-200 p-3.5 dark:border-zinc-800 dark:bg-zinc-900/50"
              >
                <div>
                  <div className="flex items-center gap-2">
                    <h3 className="text-sm font-medium text-zinc-900 dark:text-zinc-100">{rule.name}</h3>
                    <span className="rounded bg-zinc-100 px-2 py-0.5 text-[11px] font-mono text-zinc-600 dark:bg-zinc-800 dark:text-zinc-400">
                      {rule.channel}
                    </span>
                  </div>
                  <p className="mt-0.5 text-xs text-zinc-500">{rule.condition}</p>
                </div>
                <button
                  type="button"
                  onClick={() => toggleRule(rule.id)}
                  className={`rounded-md border px-3 py-1 text-xs font-medium ${
                    rule.enabled
                      ? "border-emerald-500/40 bg-emerald-500/10 text-emerald-400"
                      : "border-zinc-200 bg-zinc-50 text-zinc-500 dark:border-zinc-800 dark:bg-zinc-900"
                  }`}
                >
                  {rule.enabled ? "Active ✓" : "Disabled"}
                </button>
              </div>
            ))}
          </div>
        </section>

        {/* Webhook Endpoint Integration */}
        <section className="rounded-lg border border-zinc-200 bg-white p-5 dark:border-zinc-800 dark:bg-zinc-950">
          <h2 className="text-sm font-semibold text-zinc-900 dark:text-zinc-100">Webhook Notification Dispatcher</h2>
          <p className="mt-1 text-sm text-zinc-500 dark:text-zinc-400 mb-3">
            Receive instant JSON payloads in Slack, Discord, or PagerDuty when an SLA breach occurs.
          </p>
          <form onSubmit={handleSaveWebhook} className="flex gap-3">
            <input
              type="text"
              value={webhookUrl}
              onChange={(e) => setWebhookUrl(e.target.value)}
              className="flex-1 rounded-md border border-zinc-300 bg-white px-3 py-1.5 font-mono text-sm dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-100"
            />
            <button
              type="submit"
              className="rounded-md bg-zinc-900 px-4 py-1.5 text-xs font-medium text-white hover:bg-zinc-800 dark:bg-zinc-100 dark:text-zinc-900 dark:hover:bg-zinc-200"
            >
              Save Webhook
            </button>
          </form>
        </section>

        {/* Historical SLA Event Logs */}
        <section className="rounded-lg border border-zinc-200 bg-white p-5 dark:border-zinc-800 dark:bg-zinc-950">
          <h2 className="text-sm font-semibold text-zinc-900 dark:text-zinc-100 mb-4">SLA & Fallback Alert Event Log</h2>
          <div className="overflow-hidden rounded-md border border-zinc-200 dark:border-zinc-800">
            <table className="min-w-full text-left text-xs">
              <thead className="border-b border-zinc-200 bg-zinc-50 font-medium text-zinc-500 dark:border-zinc-800 dark:bg-zinc-900">
                <tr>
                  <th className="px-3 py-2">Time</th>
                  <th className="px-3 py-2">Event Type</th>
                  <th className="px-3 py-2">Severity</th>
                  <th className="px-3 py-2">Description</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-zinc-200 dark:divide-zinc-800 font-mono">
                {events.map((evt) => (
                  <tr key={evt.id} className="hover:bg-zinc-50 dark:hover:bg-zinc-900/50">
                    <td className="px-3 py-2 text-zinc-500 whitespace-nowrap">{evt.time}</td>
                    <td className="px-3 py-2 text-zinc-900 dark:text-zinc-100 font-semibold">{evt.type}</td>
                    <td className="px-3 py-2">
                      <span
                        className={`inline-flex rounded-full px-2 py-0.5 text-[10px] uppercase tracking-wider font-semibold ${
                          evt.severity === "critical"
                            ? "bg-red-500/15 text-red-400"
                            : evt.severity === "warning"
                            ? "bg-amber-500/15 text-amber-400"
                            : "bg-blue-500/15 text-blue-400"
                        }`}
                      >
                        {evt.severity}
                      </span>
                    </td>
                    <td className="px-3 py-2 text-zinc-600 dark:text-zinc-300 font-sans">{evt.message}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>
      </div>
    </>
  );
}

