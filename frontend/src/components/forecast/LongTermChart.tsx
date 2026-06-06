import ReactECharts from "echarts-for-react";

interface Point {
  date: string;
  balance: string;
  balance_p10?: string;
  balance_p90?: string;
}

interface CompareSeries {
  series: Point[];
  label: string;
  color: string;
}

export function LongTermChart({
  series,
  compareSeries,
  showBands = false,
  lowConfidence = false,
}: {
  series?: Point[];
  compareSeries?: CompareSeries[];
  showBands?: boolean;
  lowConfidence?: boolean;
}) {
  const bandOpacity = lowConfidence ? 0.08 : 0.18;

  if (compareSeries && compareSeries.length > 0) {
    const dates = compareSeries[0]?.series.map((p) => p.date) ?? [];
    const echartsSeries = compareSeries.flatMap((cs) => {
      const lines = [
        {
          name: cs.label,
          type: "line" as const,
          smooth: true,
          data: cs.series.map((p) => parseFloat(p.balance)),
          lineStyle: { color: cs.color },
          itemStyle: { color: cs.color },
        },
      ];
      if (cs.label.includes("ML") && cs.series.some((p) => p.balance_p10)) {
        lines.push({
          name: `${cs.label} band`,
          type: "line" as const,
          smooth: true,
          data: cs.series.map((p) =>
            p.balance_p10 ? parseFloat(p.balance_p10) : parseFloat(p.balance),
          ),
          lineStyle: { opacity: 0 },
          itemStyle: { opacity: 0 },
          areaStyle: { opacity: 0 },
        } as never);
      }
      return lines;
    });

    const mlSeries = compareSeries.find((c) => c.label.includes("ML"));
    if (mlSeries?.series.some((p) => p.balance_p10 && p.balance_p90)) {
      echartsSeries.push({
        name: "ML band",
        type: "line" as const,
        smooth: true,
        stack: "band",
        data: mlSeries.series.map((p) => parseFloat(p.balance_p10 ?? p.balance)),
        lineStyle: { opacity: 0 },
        areaStyle: { color: "rgba(251, 146, 60, 0.15)" },
      } as never);
    }

    const option = {
      tooltip: { trigger: "axis" as const },
      legend: { data: compareSeries.map((c) => c.label) },
      xAxis: { type: "category" as const, data: dates },
      yAxis: { type: "value" as const },
      series: echartsSeries,
    };
    return <ReactECharts option={option} style={{ height: 400 }} notMerge lazyUpdate />;
  }

  const data = series ?? [];
  const echartsSeries: object[] = [
    {
      name: "Balance",
      type: "line" as const,
      smooth: true,
      data: data.map((p) => parseFloat(p.balance)),
      areaStyle: showBands ? undefined : { opacity: 0.12 },
      z: 2,
    },
  ];

  if (showBands && data.some((p) => p.balance_p10 && p.balance_p90)) {
    echartsSeries.unshift({
      name: "p10-p90",
      type: "line" as const,
      smooth: true,
      data: data.map((p) => parseFloat(p.balance_p10 ?? p.balance)),
      lineStyle: { opacity: 0 },
      stack: "confidence",
      symbol: "none",
      areaStyle: { color: `rgba(59, 130, 246, ${bandOpacity})` },
      z: 1,
    });
    echartsSeries.push({
      name: "p90 upper",
      type: "line" as const,
      smooth: true,
      data: data.map((p) => parseFloat(p.balance_p90 ?? p.balance)),
      lineStyle: { opacity: 0 },
      stack: "confidence",
      symbol: "none",
      areaStyle: { color: `rgba(59, 130, 246, ${bandOpacity})` },
      z: 1,
    });
  }

  const option = {
    tooltip: { trigger: "axis" as const },
    xAxis: { type: "category" as const, data: data.map((p) => p.date) },
    yAxis: { type: "value" as const },
    series: echartsSeries,
  };

  return <ReactECharts option={option} style={{ height: 400 }} notMerge lazyUpdate />;
}
