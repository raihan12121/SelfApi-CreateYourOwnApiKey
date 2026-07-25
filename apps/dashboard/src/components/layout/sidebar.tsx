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
    <aside className="flex w-64 shrink-0 flex-col border-r border-zinc-200 bg-zinc-50 dark:border-zinc-800 dark:bg-zinc-950">
      <div className="border-b border-zinc-200 px-5 py-4 dark:border-zinc-800">
        <Link href="/" className="block">
          <span className="text-lg font-semibold tracking-tight">SelfAPI</span>
          <span className="mt-0.5 block text-xs text-zinc-500 dark:text-zinc-400">
            Turn your GPU into your own AI API
          </span>
        </Link>
      </div>

      <nav className="flex-1 space-y-1 px-3 py-4">
        {primaryNav.map((item) => {
          const active = isActive(pathname, item.href);
          return (
            <Link
              key={item.href}
              href={item.href}
              className={`block rounded-md px-3 py-2 text-sm transition-colors ${
                active
                  ? "bg-zinc-900 text-white dark:bg-zinc-100 dark:text-zinc-900"
                  : "text-zinc-700 hover:bg-zinc-100 dark:text-zinc-300 dark:hover:bg-zinc-900"
              }`}
            >
              <span className="font-medium">{item.label}</span>
              {item.description ? (
                <span
                  className={`mt-0.5 block text-xs ${
                    active
                      ? "text-zinc-300 dark:text-zinc-600"
                      : "text-zinc-500 dark:text-zinc-500"
                  }`}
                >
                  {item.description}
                </span>
              ) : null}
            </Link>
          );
        })}
      </nav>
    </aside>
  );
}
