export function PrivacyBadge({ allowRaw }: { allowRaw: boolean }) {
  if (allowRaw) return null;
  return (
    <span
      className="badge"
      style={{ background: "#dcfce7", color: "#166534", fontSize: "0.75rem" }}
      title="Transaction tool returns category/month aggregates only"
    >
      Privacy: aggregates only
    </span>
  );
}
