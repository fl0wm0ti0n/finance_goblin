import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  apiFetch,
  entityCountEntries,
  EntityCounts,
  ExchangeListItem,
  SyncRun,
  SyncStatus,
  TriggerResponse,
} from "../lib/api";

function statusClass(status: string) {
  if (status === "success") return "status-success";
  if (status === "failed") return "status-failed";
  if (status === "running") return "status-running";
  return "";
}

function phaseLabel(phase: string | null | undefined) {
  if (phase === "sync") return "Syncing Firefly data…";
  if (phase === "subscriptions") return "Detecting subscriptions…";
  if (phase === "forecast") return "Recomputing forecasts…";
  if (phase === "exchanges") return "Syncing exchanges…";
  if (phase === "alerts") return "Evaluating alerts…";
  return null;
}

export function SyncStatusPage() {
  const queryClient = useQueryClient();
  const statusQuery = useQuery({
    queryKey: ["sync-status"],
    queryFn: () => apiFetch<SyncStatus>("/api/v1/sync/status"),
    refetchInterval: (q) => (q.state.data?.state === "running" ? 2000 : false),
  });
  const entitiesQuery = useQuery({
    queryKey: ["sync-entities"],
    queryFn: () => apiFetch<EntityCounts>("/api/v1/sync/entities"),
  });
  const runsQuery = useQuery({
    queryKey: ["sync-runs"],
    queryFn: () => apiFetch<SyncRun[]>("/api/v1/sync/runs"),
  });

  const exchangesQuery = useQuery({
    queryKey: ["exchanges"],
    queryFn: () => apiFetch<ExchangeListItem[]>("/api/v1/exchanges"),
  });

  const triggerMutation = useMutation({
    mutationFn: () =>
      apiFetch<TriggerResponse>("/api/v1/sync/trigger", { method: "POST" }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["sync-status"] });
      queryClient.invalidateQueries({ queryKey: ["sync-runs"] });
      queryClient.invalidateQueries({ queryKey: ["sync-entities"] });
      queryClient.invalidateQueries({ queryKey: ["forecast-meta"] });
      queryClient.invalidateQueries({ queryKey: ["subscriptions"] });
      queryClient.invalidateQueries({ queryKey: ["subscription-alerts"] });
      queryClient.invalidateQueries({ queryKey: ["wealth"] });
      queryClient.invalidateQueries({ queryKey: ["wealth-history"] });
      queryClient.invalidateQueries({ queryKey: ["alerts"] });
      queryClient.invalidateQueries({ queryKey: ["alerts-unread"] });
    },
  });

  const exchangeTriggerMutation = useMutation({
    mutationFn: () =>
      apiFetch<TriggerResponse>("/api/v1/sync/exchanges/trigger", { method: "POST" }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["sync-status"] });
      queryClient.invalidateQueries({ queryKey: ["exchanges"] });
      queryClient.invalidateQueries({ queryKey: ["wealth"] });
    },
  });

  const isRunning = statusQuery.data?.state === "running";
  const phaseText = phaseLabel(statusQuery.data?.phase);
  const lastRun = statusQuery.data?.last_run;

  return (
    <div>
      <div
        className="card"
        style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}
      >
        <div>
          <h1>Sync Status</h1>
          <p>
            Last sync:{" "}
            {lastRun?.finished_at
              ? new Date(lastRun.finished_at).toLocaleString()
              : "Never"}
          </p>
          {isRunning && phaseText && (
            <p className="status-running" style={{ margin: "0.25rem 0 0" }}>
              {phaseText}
            </p>
          )}
        </div>
        <button
          className="btn"
          disabled={isRunning || triggerMutation.isPending}
          onClick={() => triggerMutation.mutate()}
        >
          {isRunning ? "Sync running…" : "Sync now"}
        </button>
      </div>

      {triggerMutation.isError && (
        <div className="card" style={{ color: "#b91c1c" }}>
          {(triggerMutation.error as Error).message}
        </div>
      )}

      <div className="grid">
        {entityCountEntries(entitiesQuery.data).map((row) => (
          <div key={row.entity} className="card">
            <div style={{ textTransform: "capitalize" }}>{row.entity.replace("_", " ")}</div>
            <div className="stat">{row.count}</div>
          </div>
        ))}
      </div>

      {(exchangesQuery.data?.length ?? 0) > 0 && (
        <div className="card" style={{ marginTop: "1rem" }}>
          <div
            style={{
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center",
              marginBottom: "0.75rem",
            }}
          >
            <h2 style={{ margin: 0 }}>Exchange sync</h2>
            <button
              type="button"
              className="btn"
              disabled={isRunning || exchangeTriggerMutation.isPending}
              onClick={() => exchangeTriggerMutation.mutate()}
            >
              Sync exchanges now
            </button>
          </div>
          <table>
            <thead>
              <tr>
                <th>Exchange</th>
                <th>Status</th>
                <th>Last sync</th>
                <th>Holdings</th>
                <th>Trades</th>
                <th>Error</th>
              </tr>
            </thead>
            <tbody>
              {(exchangesQuery.data ?? []).map((ex) => (
                <tr key={ex.id}>
                  <td style={{ textTransform: "capitalize" }}>{ex.id}</td>
                  <td>{ex.connection_state}</td>
                  <td>
                    {ex.last_sync_at ? new Date(ex.last_sync_at).toLocaleString() : "—"}
                  </td>
                  <td>{ex.counts.holdings}</td>
                  <td>{ex.counts.trades}</td>
                  <td style={{ color: "#b91c1c" }}>{ex.last_error ?? "—"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      <div className="card">
        <h2>Sync history</h2>
        <table>
          <thead>
            <tr>
              <th>Started</th>
              <th>Finished</th>
              <th>Status</th>
              <th>Trigger</th>
            </tr>
          </thead>
          <tbody>
            {(runsQuery.data ?? []).map((run: SyncRun) => (
              <tr key={run.id}>
                <td>{new Date(run.started_at).toLocaleString()}</td>
                <td>{run.finished_at ? new Date(run.finished_at).toLocaleString() : "—"}</td>
                <td className={statusClass(run.status)}>{run.status}</td>
                <td>{run.trigger}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
