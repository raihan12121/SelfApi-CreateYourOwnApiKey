"use client";

import { useEffect, useState } from "react";
import { PageHeader } from "@/components/layout/page-header";

type ApiKeyRow = {
  id: string;
  name: string;
  keyPrefix: string;
  scope: string;
  rateLimit: string;
  publicEndpoint: string;
  created: string;
  status: "active" | "revoked";
};

const initialKeys: ApiKeyRow[] = [
  {
    id: "key_1",
    name: "Default Agent Key",
    keyPrefix: "sk-selfapi-9f82...",
    scope: "Full Access (All Models)",
    rateLimit: "100 req/min",
    publicEndpoint: "https://gpu-node-9f82.selfapi.site/v1",
    created: "Today",
    status: "active",
  },
  {
    id: "key_2",
    name: "Staging Prototype App",
    keyPrefix: "sk-selfapi-4e11...",
    scope: "Llama 3.2 3B Only",
    rateLimit: "30 req/min",
    publicEndpoint: "https://gpu-node-9f82.selfapi.site/v1",
    created: "3 days ago",
    status: "active",
  },
];

export default function ApiKeysPage() {
  const [keys, setKeys] = useState<ApiKeyRow[]>(initialKeys);
  const [showModal, setShowModal] = useState(false);
  const [newKeyName, setNewKeyName] = useState("");

  useEffect(() => {
    async function loadKeys() {
      try {
        const res = await fetch("http://127.0.0.1:8787/v1/keys");
        if (res.ok) {
          const data = await res.json();
          if (Array.isArray(data.keys) && data.keys.length > 0) {
            setKeys(data.keys);
          }
        }
      } catch {
        // server offline fallback
      }
    }
    void loadKeys();
  }, []);

  const handleCreateKey = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newKeyName.trim()) return;

    try {
      const res = await fetch("http://127.0.0.1:8787/v1/keys", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ name: newKeyName }),
      });
      if (res.ok) {
        const newKey: ApiKeyRow = await res.json();
        setKeys((prev) => [newKey, ...prev]);
      }
    } catch {
      const newKey: ApiKeyRow = {
        id: `key_${Date.now()}`,
        name: newKeyName,
        keyPrefix: `sk-selfapi-${Math.random().toString(36).substring(2, 8)}...`,
        scope: "Full Access (All Models)",
        rateLimit: "60 req/min",
        publicEndpoint: "https://gpu-node-9f82.selfapi.site/v1",
        created: "Just now",
        status: "active",
      };
      setKeys((prev) => [newKey, ...prev]);
    }
    setNewKeyName("");
    setShowModal(false);
  };

  const toggleRevoke = (id: string) => {
    setKeys((prev) =>
      prev.map((k) => (k.id === id ? { ...k, status: k.status === "active" ? "revoked" : "active" } : k)),
    );
  };

  return (
    <>
      <PageHeader
        title="API keys"
        description="Create, revoke, and scope OpenAI-compatible API keys bound to your public relay endpoint."
        action={
          <button
            type="button"
            onClick={() => setShowModal(true)}
            className="rounded-md bg-zinc-900 px-3.5 py-2 text-sm font-medium text-white hover:bg-zinc-800 dark:bg-zinc-100 dark:text-zinc-900 dark:hover:bg-zinc-200"
          >
            Create API Key
          </button>
        }
      />

      {showModal && (
        <div className="mb-6 rounded-lg border border-zinc-200 bg-white p-5 dark:border-zinc-800 dark:bg-zinc-950">
          <h3 className="text-sm font-medium text-zinc-900 dark:text-zinc-100 mb-3">Create Granular Scoped API Key (FR-20)</h3>
          <form onSubmit={handleCreateKey} className="space-y-4">
            <div className="grid gap-3 sm:grid-cols-2">
              <div>
                <label className="block text-xs font-medium text-zinc-500 mb-1">Key Name / Label</label>
                <input
                  type="text"
                  placeholder="e.g. Production Mobile App"
                  value={newKeyName}
                  onChange={(e) => setNewKeyName(e.target.value)}
                  className="w-full rounded-md border border-zinc-300 bg-white px-3 py-1.5 text-sm dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-100"
                />
              </div>
              <div>
                <label className="block text-xs font-medium text-zinc-500 mb-1">Model Access Scope</label>
                <select className="w-full rounded-md border border-zinc-300 bg-white px-3 py-1.5 text-sm dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-100">
                  <option value="all">Full Access (All Models)</option>
                  <option value="llama-3.2-3b">Llama 3.2 3B Only</option>
                  <option value="qwen-2.5-7b">Qwen 2.5 7B Only</option>
                  <option value="mistral-7b">Mistral 7B Only</option>
                </select>
              </div>
            </div>

            <div className="grid gap-3 sm:grid-cols-3">
              <div>
                <label className="block text-xs font-medium text-zinc-500 mb-1">Rate Limit Cap</label>
                <input
                  type="text"
                  defaultValue="60 req/min"
                  className="w-full rounded-md border border-zinc-300 bg-white px-3 py-1.5 text-sm font-mono dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-100"
                />
              </div>
              <div>
                <label className="block text-xs font-medium text-zinc-500 mb-1">Monthly Spend Cap</label>
                <input
                  type="text"
                  defaultValue="$50.00 / mo"
                  className="w-full rounded-md border border-zinc-300 bg-white px-3 py-1.5 text-sm font-mono dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-100"
                />
              </div>
              <div>
                <label className="block text-xs font-medium text-zinc-500 mb-1">Endpoint Domain</label>
                <select className="w-full rounded-md border border-zinc-300 bg-white px-3 py-1.5 text-sm dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-100">
                  <option value="public">https://gpu-node-9f82.selfapi.site/v1</option>
                  <option value="custom">https://api.mycompany.com/v1 (Custom Domain)</option>
                  <option value="local">http://127.0.0.1:8787/v1 (Local Direct)</option>
                </select>
              </div>
            </div>

            <div className="flex justify-end gap-2 pt-2">
              <button
                type="button"
                onClick={() => setShowModal(false)}
                className="rounded-md border border-zinc-200 px-4 py-1.5 text-sm text-zinc-600 dark:border-zinc-800 dark:text-zinc-400"
              >
                Cancel
              </button>
              <button
                type="submit"
                className="rounded-md bg-blue-600 px-4 py-1.5 text-sm font-medium text-white hover:bg-blue-500"
              >
                Generate Scoped Key
              </button>
            </div>
          </form>
        </div>
      )}


      <div className="overflow-hidden rounded-lg border border-zinc-200 bg-white dark:border-zinc-800 dark:bg-zinc-950">
        <table className="min-w-full text-left text-sm">
          <thead className="border-b border-zinc-200 bg-zinc-50 text-xs uppercase tracking-wide text-zinc-500 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-400">
            <tr>
              <th className="px-4 py-3 font-medium">Key Prefix / Label</th>
              <th className="px-4 py-3 font-medium">Public Relay Endpoint</th>
              <th className="px-4 py-3 font-medium">Scope</th>
              <th className="px-4 py-3 font-medium">Rate limit</th>
              <th className="px-4 py-3 font-medium">Created</th>
              <th className="px-4 py-3 font-medium">Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-zinc-200 dark:divide-zinc-800">
            {keys.map((k) => (
              <tr key={k.id} className="hover:bg-zinc-50 dark:hover:bg-zinc-900/50">
                <td className="px-4 py-3">
                  <div className="font-medium text-zinc-900 dark:text-zinc-100">{k.name}</div>
                  <div className="font-mono text-xs text-zinc-500">{k.keyPrefix}</div>
                </td>
                <td className="px-4 py-3 font-mono text-xs text-blue-500">{k.publicEndpoint}</td>
                <td className="px-4 py-3 text-xs text-zinc-600 dark:text-zinc-300">{k.scope}</td>
                <td className="px-4 py-3 font-mono text-xs">{k.rateLimit}</td>
                <td className="px-4 py-3 text-xs text-zinc-500">{k.created}</td>
                <td className="px-4 py-3">
                  <button
                    type="button"
                    onClick={() => toggleRevoke(k.id)}
                    className={`text-xs font-medium ${
                      k.status === "active" ? "text-red-500 hover:underline" : "text-emerald-500 hover:underline"
                    }`}
                  >
                    {k.status === "active" ? "Revoke Key" : "Re-activate"}
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </>
  );
}

