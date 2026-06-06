import ReactECharts from "echarts-for-react";

interface PriceEvent {
  occurred_at: string;
  amount: string;
  event_type: string;
}

export function PriceHistoryChart({ events }: { events: PriceEvent[] }) {
  const billing = events.filter((e) => e.event_type === "billing" || e.event_type.startsWith("price_"));

  if (billing.length === 0) {
    return <p>No price history yet.</p>;
  }

  const dates = billing.map((e) => e.occurred_at);
  const amounts = billing.map((e) => Math.abs(Number(e.amount)));

  const option = {
    tooltip: { trigger: "axis" as const },
    xAxis: { type: "category" as const, data: dates },
    yAxis: { type: "value" as const, name: "€" },
    series: [
      {
        type: "line" as const,
        data: amounts,
        smooth: true,
        markPoint: {
          data: billing
            .map((e, i) =>
              e.event_type.includes("increase") || e.event_type.includes("decrease")
                ? { coord: [dates[i], amounts[i]], name: e.event_type }
                : null,
            )
            .filter(Boolean),
        },
      },
    ],
  };

  return <ReactECharts option={option} style={{ height: 280 }} />;
}
