const ACCOUNT_ROLE_LABELS: Record<string, string> = {
  defaultAsset: "Checking",
  cashWalletAsset: "Cash wallet",
  savingAsset: "Savings",
  sharedAsset: "Shared",
  ccAsset: "Credit card",
};

export function formatAccountRole(role: string | null | undefined): string {
  if (!role) return "—";
  return ACCOUNT_ROLE_LABELS[role] ?? role;
}
