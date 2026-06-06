import type { WealthHistoryPoint } from "../../lib/api";

interface Props {
  data: WealthHistoryPoint[];
}

export function WealthChart({ data }: Props) {
  // Simple SVG line chart — avoids heavy ECharts for trend-only view
  if (data.length === 0) return null;

  const width = 640;
  const height = 240;
  const padding = 40;
  const values = data.map((d) => d.total_eur);
  const min = Math.min(...values);
  const max = Math.max(...values);
  const range = max - min || 1;

  const points = data
    .map((d, i) => {
      const x = padding + (i / Math.max(data.length - 1, 1)) * (width - padding * 2);
      const y = height - padding - ((d.total_eur - min) / range) * (height - padding * 2);
      return `${x},${y}`;
    })
    .join(" ");

  return (
    <svg viewBox={`0 0 ${width} ${height}`} style={{ width: "100%", maxWidth: width }}>
      <polyline
        fill="none"
        stroke="#2563eb"
        strokeWidth="2"
        points={points}
      />
      <text x={padding} y={20} fontSize="12" fill="#64748b">
        €{max.toLocaleString()}
      </text>
      <text x={padding} y={height - 10} fontSize="12" fill="#64748b">
        €{min.toLocaleString()}
      </text>
    </svg>
  );
}
