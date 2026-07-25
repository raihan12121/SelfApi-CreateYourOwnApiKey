type StatusVariant = "online" | "offline" | "fallback" | "success" | "error" | "warning";

const variantStyles: Record<StatusVariant, string> = {
  online: "bg-emerald-500/15 text-emerald-700 dark:text-emerald-300",
  success: "bg-emerald-500/15 text-emerald-700 dark:text-emerald-300",
  offline: "bg-red-500/15 text-red-700 dark:text-red-300",
  error: "bg-red-500/15 text-red-700 dark:text-red-300",
  fallback: "bg-amber-500/15 text-amber-800 dark:text-amber-300",
  warning: "bg-amber-500/15 text-amber-800 dark:text-amber-300",
};

const variantLabels: Record<StatusVariant, string> = {
  online: "Online",
  success: "Success",
  offline: "Offline",
  error: "Error",
  fallback: "Fallback",
  warning: "Warning",
};

export function StatusBadge({
  variant,
  label,
}: {
  variant: StatusVariant;
  label?: string;
}) {
  return (
    <span
      className={`inline-flex items-center gap-1.5 rounded-md px-2 py-0.5 text-xs font-medium ${variantStyles[variant]}`}
    >
      <span
        aria-hidden="true"
        className={`h-1.5 w-1.5 rounded-full ${
          variant === "online" || variant === "success"
            ? "bg-emerald-500"
            : variant === "fallback" || variant === "warning"
              ? "bg-amber-500"
              : "bg-red-500"
        }`}
      />
      {label ?? variantLabels[variant]}
    </span>
  );
}
