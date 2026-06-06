interface ProviderBadgeProps {
  label: string;
  isLocal: boolean;
}

export function ProviderBadge({ label, isLocal }: ProviderBadgeProps) {
  return (
    <span
      className="badge"
      style={{
        background: isLocal ? "#e0e7ff" : "#dbeafe",
        color: isLocal ? "#3730a3" : "#1e40af",
      }}
      title={isLocal ? "Local or self-hosted AI" : "Cloud AI provider"}
    >
      {label}
    </span>
  );
}
