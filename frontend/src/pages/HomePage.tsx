import { Link } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { apiFetch, entityCountEntries, EntityCounts, SyncStatus } from "../lib/api";

export function HomePage() {
  const statusQuery = useQuery({
    queryKey: ["sync-status"],
    queryFn: () => apiFetch<SyncStatus>("/api/v1/sync/status"),
  });
  const entitiesQuery = useQuery({
    queryKey: ["sync-entities"],
    queryFn: () => apiFetch<EntityCounts>("/api/v1/sync/entities"),
  });

  const totalRecords = entityCountEntries(entitiesQuery.data).reduce(
    (sum, row) => sum + row.count,
    0,
  );

  return (
    <div>
      <div className="card">
        <h1>Welcome to Flow Finance AI</h1>
        <p>
          Self-hosted analytics on your Firefly III data. This platform syncs
          read-only from Firefly and never writes back to your ledger.
        </p>
        <p>
          <Link to="/sync">View Sync Status</Link>
          {" · "}
          <Link to="/settings">Settings</Link>
        </p>
      </div>

      <div className="grid">
        <div className="card">
          <div>Sync state</div>
          <div className="stat">{statusQuery.data?.state ?? "—"}</div>
        </div>
        <div className="card">
          <div>Mirrored records</div>
          <div className="stat">{totalRecords}</div>
        </div>
        <div className="card">
          <div>Last sync</div>
          <div className="stat" style={{ fontSize: "1rem" }}>
            {statusQuery.data?.last_run?.finished_at
              ? new Date(statusQuery.data.last_run.finished_at).toLocaleString()
              : "Never"}
          </div>
        </div>
      </div>
    </div>
  );
}
