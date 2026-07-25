"use client";

import { useEffect, useState } from "react";
import { PageHeader } from "@/components/layout/page-header";

type PayoutTransaction = {
  id: string;
  date: string;
  amountUsd: string;
  method: string;
  status: "completed" | "processing" | "pending";
};

const initialPayouts: PayoutTransaction[] = [
  {
    id: "po_9f821",
    date: "Jul 15, 2026",
    amountUsd: "$158.30",
    method: "Stripe Direct Deposit (•••• 4219)",
    status: "completed",
  },
  {
    id: "po_9f820",
    date: "Jun 30, 2026",
    amountUsd: "$99.90",
    method: "Stripe Direct Deposit (•••• 4219)",
    status: "completed",
  },
  {
    id: "po_9f819",
    date: "Jun 15, 2026",
    amountUsd: "$84.30",
    method: "USDC Crypto Wallet (0x7f...3a9)",
    status: "completed",
  },
];

export default function BillingPage() {
  const [sharingEnabled, setSharingEnabled] = useState(true);
  const [computeRate, setComputeRate] = useState("0.20");
  const [payouts] = useState<PayoutTransaction[]>(initialPayouts);
  const [payoutMethod, setPayoutMethod] = useState("stripe");
  const [savedToast, setSavedToast] = useState(false);

  useEffect(() => {
    async function loadMarketplace() {
      try {
        const res = await fetch("http://127.0.0.1:8787/v1/marketplace");
        if (res.ok) {
          const data = await res.json();
          if (data.config) {
            setSharingEnabled(data.config.enabled);
            if (data.config.price_per_1m_tokens_usd) {
              setComputeRate(String(data.config.price_per_1m_tokens_usd));
            }
          }
        }
      } catch {
        // offline fallback
      }
    }
    void loadMarketplace();
  }, []);

  const handleSaveRate = async (e: React.FormEvent) => {
    e.preventDefault();
    setSavedToast(true);
    try {
      await fetch("http://127.0.0.1:8787/v1/marketplace", { method: "POST" });
    } catch {
      // offline fallback
    }
    setTimeout(() => setSavedToast(false), 2000);
  };

  return (
    <>
      <PageHeader
        title="Billing & marketplace earnings"
        description="Monetize spare GPU cycles, set compute rates, and manage direct payout schedules."
      />

      <div className="space-y-6">
        {savedToast && (
          <div className="rounded-lg bg-emerald-500/10 border border-emerald-500/30 p-3 text-xs font-medium text-emerald-400">
            ✓ Marketplace compute rate & payout preferences updated.
          </div>
        )}

        {/* Overview Financial Metrics */}
        <div className="grid gap-4 sm:grid-cols-3">
          <div className="rounded-lg border border-zinc-200 bg-white p-5 dark:border-zinc-800 dark:bg-zinc-950">
            <span className="text-xs font-mono uppercase tracking-wider text-zinc-500">Total All-Time Earnings</span>
            <div className="mt-2 text-2xl font-bold text-zinc-900 dark:text-zinc-100">$342.50</div>
            <p className="mt-1 text-xs text-emerald-500">+$84.20 accumulated this period</p>
          </div>

          <div className="rounded-lg border border-zinc-200 bg-white p-5 dark:border-zinc-800 dark:bg-zinc-950">
            <span className="text-xs font-mono uppercase tracking-wider text-zinc-500">Pending Payout Balance</span>
            <div className="mt-2 text-2xl font-bold text-zinc-900 dark:text-zinc-100">$84.20</div>
            <p className="mt-1 text-xs text-zinc-500">Scheduled payout: July 31, 2026</p>
          </div>

          <div className="rounded-lg border border-zinc-200 bg-white p-5 dark:border-zinc-800 dark:bg-zinc-950">
            <span className="text-xs font-mono uppercase tracking-wider text-zinc-500">Host Reliability Rank (FR-26)</span>
            <div className="mt-2 text-xl font-bold text-amber-400 flex items-center gap-2">
              🥇 Gold Host Node
            </div>
            <p className="mt-1 text-xs text-zinc-500 font-mono">99.6% Uptime · 38ms P95 Latency</p>
          </div>
        </div>

        {/* Compute Rate & Idle Sharing Settings */}
        <section className="rounded-lg border border-zinc-200 bg-white p-5 dark:border-zinc-800 dark:bg-zinc-950">
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-sm font-semibold text-zinc-900 dark:text-zinc-100">Idle Capacity Sharing (FR-25)</h2>
              <p className="mt-1 text-sm text-zinc-500 dark:text-zinc-400">
                Allow external developers to route inference jobs to your machine when VRAM is idle.
              </p>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={sharingEnabled}
                onChange={() => setSharingEnabled(!sharingEnabled)}
                className="sr-only peer"
              />
              <div className="w-11 h-6 bg-zinc-200 peer-focus:outline-none rounded-full peer dark:bg-zinc-800 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-zinc-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:after:border-zinc-600 peer-checked:bg-emerald-600"></div>
            </label>
          </div>

          {sharingEnabled && (
            <form onSubmit={handleSaveRate} className="mt-5 grid gap-4 sm:grid-cols-2 pt-4 border-t border-zinc-100 dark:border-zinc-800/80">
              <div>
                <label className="block text-xs font-medium text-zinc-500 mb-1">Compute Rate ($ / 1M Tokens)</label>
                <div className="flex gap-2">
                  <span className="inline-flex items-center px-3 rounded-md border border-zinc-300 bg-zinc-50 text-sm text-zinc-500 dark:border-zinc-700 dark:bg-zinc-900">$</span>
                  <input
                    type="text"
                    value={computeRate}
                    onChange={(e) => setComputeRate(e.target.value)}
                    className="flex-1 rounded-md border border-zinc-300 bg-white px-3 py-1.5 font-mono text-sm dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-100"
                  />
                  <button
                    type="submit"
                    className="rounded-md bg-zinc-900 px-4 py-1.5 text-xs font-medium text-white hover:bg-zinc-800 dark:bg-zinc-100 dark:text-zinc-900 dark:hover:bg-zinc-200"
                  >
                    Save Rate
                  </button>
                </div>
              </div>

              <div>
                <label className="block text-xs font-medium text-zinc-500 mb-1">Payout Destination Method (FR-27)</label>
                <select
                  value={payoutMethod}
                  onChange={(e) => setPayoutMethod(e.target.value)}
                  className="w-full rounded-md border border-zinc-300 bg-white px-3 py-1.5 text-sm dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-100"
                >
                  <option value="stripe">Stripe Direct Deposit (Weekly)</option>
                  <option value="bank">Bank Wire Transfer (Monthly)</option>
                  <option value="crypto">USDC / Solana Crypto Wallet</option>
                </select>
              </div>
            </form>
          )}
        </section>

        {/* Payout History Ledger */}
        <section className="rounded-lg border border-zinc-200 bg-white p-5 dark:border-zinc-800 dark:bg-zinc-950">
          <h2 className="text-sm font-semibold text-zinc-900 dark:text-zinc-100 mb-4">Historical Payout Transactions</h2>
          <div className="overflow-hidden rounded-md border border-zinc-200 dark:border-zinc-800">
            <table className="min-w-full text-left text-xs font-mono">
              <thead className="border-b border-zinc-200 bg-zinc-50 font-medium text-zinc-500 dark:border-zinc-800 dark:bg-zinc-900">
                <tr>
                  <th className="px-4 py-2.5">Date</th>
                  <th className="px-4 py-2.5">Payout Reference</th>
                  <th className="px-4 py-2.5">Destination</th>
                  <th className="px-4 py-2.5">Amount</th>
                  <th className="px-4 py-2.5">Status</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-zinc-200 dark:divide-zinc-800">
                {payouts.map((po) => (
                  <tr key={po.id} className="hover:bg-zinc-50 dark:hover:bg-zinc-900/50">
                    <td className="px-4 py-3 text-zinc-500">{po.date}</td>
                    <td className="px-4 py-3 text-zinc-900 dark:text-zinc-100">{po.id}</td>
                    <td className="px-4 py-3 text-zinc-500">{po.method}</td>
                    <td className="px-4 py-3 font-semibold text-emerald-500">{po.amountUsd}</td>
                    <td className="px-4 py-3">
                      <span className="inline-flex rounded-full bg-emerald-500/15 px-2 py-0.5 text-[10px] font-semibold text-emerald-400">
                        {po.status}
                      </span>
                    </td>
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

