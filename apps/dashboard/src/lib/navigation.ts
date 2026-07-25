export type NavItem = {
  label: string;
  href: string;
  description?: string;
};

export const primaryNav: NavItem[] = [
  { label: "Dashboard", href: "/", description: "Overview and alerts" },
  { label: "Machines", href: "/machines", description: "Connected hardware" },
  { label: "Models", href: "/models", description: "Installed models" },
  { label: "API keys", href: "/api-keys", description: "Keys and scopes" },
  {
    label: "Request history",
    href: "/requests",
    description: "Logs and analytics",
  },
  { label: "Billing", href: "/billing", description: "Usage and earnings" },
  { label: "Alerts", href: "/alerts", description: "Notifications" },
  { label: "Settings", href: "/settings", description: "Privacy and fallback" },
];
