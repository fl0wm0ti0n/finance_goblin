import ReactECharts from "echarts-for-react";
import { useQuery } from "@tanstack/react-query";
import { fetchCategoryExpenseSeries, ExpenseSeriesResponse } from "../../lib/api";

function formatEur(amount: number): string {
  return `€${amount.toLocaleString(undefined, { minimumFractionDigits: 0, maximumFractionDigits: 0 })}`;
}

function formatMom(pct: number): string {
  const sign = pct > 0 ? "+" : "";
  return `${sign}${pct.toFixed(1)}%`;
}

export function CategoryTrendChart({
  categoryId,
  months = 12,
  title = "Category spending trend",
}: {
  categoryId: string;
  months?: number;
  title?: string;
}) {
  const seriesQuery = useQuery({
    queryKey: ["category-expense-series", categoryId, months],
    queryFn: () => fetchCategoryExpenseSeries(categoryId, months),
    enabled: !!categoryId,
  });

  if (!categoryId) {
    return (
      <div className="card">
        <h3>{title}</h3>
        <p style={{ color: "#64748b", margin: 0 }}>
          Select a category to view monthly actual spending.
        </p>
      </div>
    );
  }

  if (seriesQuery.isLoading) {
    return (
      <div className="card">
        <h3>{title}</h3>
        <p>Loading trend…</p>
      </div>
    );
  }

  if (seriesQuery.isError) {
    return (
      <div className="card">
        <h3>{title}</h3>
        <p style={{ color: "#b45309" }}>
          Could not load expense series. The category may have been removed from Firefly — run a Full
          sync or pick another category.
        </p>
      </div>
    );
  }

  const data = seriesQuery.data as ExpenseSeriesResponse;
  const displayName =
    data.category_label ?? data.category_name ?? data.category_id;

  if (data.transaction_count === 0) {
    return (
      <div className="card">
        <h3>{title}</h3>
        <p style={{ color: "#64748b", margin: 0 }}>
          No categorized spending in this period for {displayName}.
        </p>
      </div>
    );
  }

  const monthLabels = data.months.map((m) => m.month);
  const outflows = data.months.map((m) => m.outflow_eur);

  const option = {
    tooltip: {
      trigger: "axis" as const,
      formatter: (params: { name: string; value: number }[]) => {
        const p = params[0];
        return `${p.name}<br/>${formatEur(p.value)} outflow`;
      },
    },
    xAxis: { type: "category" as const, data: monthLabels },
    yAxis: { type: "value" as const },
    series: [
      {
        name: "Outflow",
        type: "bar" as const,
        data: outflows,
      },
    ],
  };

  return (
    <div className="card">
      <h3 style={{ marginTop: 0 }}>{title}</h3>
      <p style={{ fontSize: "0.9rem", color: "#475569", marginTop: 0 }}>
        Actual mirror spending for <strong>{displayName}</strong> — household forecast buckets above
        are unchanged.
      </p>
      <div className="grid" style={{ marginBottom: "1rem" }}>
        <div className="card">
          <div>MoM change</div>
          <div className="stat">{formatMom(data.summary.mom_delta_pct)}</div>
        </div>
        <div className="card">
          <div>Highest month</div>
          <div className="stat">{data.summary.best_month}</div>
        </div>
        <div className="card">
          <div>Lowest month</div>
          <div className="stat">{data.summary.worst_month}</div>
        </div>
      </div>
      <ReactECharts option={option} style={{ height: 320 }} notMerge lazyUpdate />
    </div>
  );
}
