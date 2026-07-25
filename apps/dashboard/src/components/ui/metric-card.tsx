export function MetricCard({
  label,
  value,
  detail,
}: {
  label: string;
  value: string;
  detail?: string;
}) {
  return (
    <div className="rounded-lg border border-zinc-200 bg-white p-4 dark:border-zinc-800 dark:bg-zinc-950">
      <p className="text-sm text-zinc-500 dark:text-zinc-400">{label}</p>
      <p className="mt-1 font-mono text-2xl font-semibold tracking-tight">
        {value}
      </p>
      {detail ? (
        <p className="mt-1 text-xs text-zinc-500 dark:text-zinc-400">{detail}</p>
      ) : null}
    </div>
  );
}
