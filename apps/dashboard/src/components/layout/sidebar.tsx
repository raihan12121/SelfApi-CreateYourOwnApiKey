"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { primaryNav } from "@/lib/navigation";

function isActive(pathname: string, href: string) {
  if (href === "/") return pathname === "/";
  return pathname.startsWith(href);
}

export function Sidebar() {
  const pathname = usePathname();

  return (
    <aside className="flex w-72 shrink-0 flex-col border-r border-white/10 bg-slate-950/80 backdrop-blur-xl">
      <div className="flex items-center gap-3 border-b border-white/10 px-6 py-5">
        <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-br from-indigo-500 to-cyan-500 font-bold text-white shadow-lg shadow-indigo-500/30">
          ⚡
        </div>
        <div>
          <Link href="/" className="block">
            <span className="text-lg font-extrabold tracking-tight bg-gradient-to-r from-white to-slate-400 bg-clip-text text-transparent">
              SelfAPI
            </span>
            <span className="block text-xs font-medium text-cyan-400">
              GPU AI Gateway
            </span>
          </Link>
        </div>
      </div>

      <nav className="flex-1 space-y-1.5 px-4 py-6">
        <div className="px-3 pb-2 text-[11px] font-bold uppercase tracking-wider text-slate-400">
          Dashboard Controls
        </div>
        {primaryNav.map((item) => {
          const active = isActive(pathname, item.href);
          return (
            <Link
              key={item.href}
              href={item.href}
              className={`block rounded-xl px-4 py-3 text-sm transition-all duration-200 ${
                active
                  ? "glass-nav-active text-white font-semibold"
                  : "text-slate-400 hover:bg-white/5 hover:text-white"
              }`}
            >
              <div className="font-medium text-sm">{item.label}</div>
              {item.description ? (
                <div
                  className={`mt-0.5 text-xs ${
                    active ? "text-indigo-200" : "text-slate-400"
                  }`}
                >
                  {item.description}
                </div>
              ) : null}
            </Link>
          );
        })}
      </nav>

      <div className="m-4 rounded-xl border border-white/10 bg-slate-900/80 p-4">
        <div className="flex items-center justify-between text-xs text-slate-400">
          <span>Agent Gateway</span>
          <span className="font-mono text-cyan-400">Port 8787</span>
        </div>
        <div className="mt-2 flex items-center gap-2 text-xs font-semibold text-emerald-400">
          <span className="h-2 w-2 rounded-full bg-emerald-400 shadow-[0_0_8px_#34d399] animate-pulse" />
          <span>Server Active</span>
        </div>
      </div>
    </aside>
  );
}
