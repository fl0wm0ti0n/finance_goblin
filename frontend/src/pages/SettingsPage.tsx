import { useState } from "react";

import { useMutation, useQuery } from "@tanstack/react-query";

import { apiFetch, AiAuditRow, AiTestResponse, Settings } from "../lib/api";



export function SettingsPage() {

  const [auditOffset, setAuditOffset] = useState(0);

  const limit = 25;



  const settingsQuery = useQuery({

    queryKey: ["settings"],

    queryFn: () => apiFetch<Settings>("/api/v1/settings"),

  });



  const auditQuery = useQuery({

    queryKey: ["ai-audit", auditOffset],

    queryFn: () =>

      apiFetch<AiAuditRow[]>(`/api/v1/ai/audit?limit=${limit}&offset=${auditOffset}`),

  });



  const s = settingsQuery.data;

  const ai = s?.ai;

  const exchanges = s?.exchanges;

  const providerConfigured =

    s?.provider_configured ?? s?.openai_configured ?? false;



  const exchangeTestMutation = useMutation({

    mutationFn: (id: string) =>

      apiFetch<{ ok: boolean; message: string; latency_ms: number }>(

        `/api/v1/exchanges/${id}/test`,

        { method: "POST" },

      ),

  });



  const aiTestMutation = useMutation({

    mutationFn: () =>

      apiFetch<AiTestResponse>("/api/v1/ai/test", {

        method: "POST",

        body: JSON.stringify({ prompt: "Reply OK." }),

      }),

  });



  const exchangeRows: { id: string; label: string; cfg: NonNullable<typeof exchanges>["binance"] }[] =

    exchanges

      ? [

          { id: "binance", label: "Binance", cfg: exchanges.binance },

          { id: "bybit", label: "Bybit", cfg: exchanges.bybit },

          { id: "bitunix", label: "Bitunix", cfg: exchanges.bitunix },

        ]

      : [];



  return (

    <div className="card">

      <h1>Settings</h1>

      <p>Non-secret operator configuration (read-only display).</p>

      <table>

        <tbody>

          <tr>

            <th>Database mode</th>

            <td>{s?.database_mode ?? "—"}</td>

          </tr>

          <tr>

            <th>Firefly URL</th>

            <td>{s?.firefly_base_url ?? "—"}</td>

          </tr>

          <tr>

            <th>Firefly auth</th>

            <td>{s?.firefly_auth_method ?? "—"}</td>

          </tr>

          <tr>

            <th>Sync interval (seconds)</th>

            <td>{s?.sync_interval_seconds ?? "—"}</td>

          </tr>

          <tr>

            <th>OIDC issuer</th>

            <td>{s?.oidc_issuer_url || "Not configured"}</td>

          </tr>

          <tr>

            <th>Firefly access</th>

            <td>{s?.read_only ? "Read-only (GET only)" : "—"}</td>

          </tr>

        </tbody>

      </table>



      <h2 style={{ marginTop: "2rem" }}>AI &amp; Privacy</h2>

      <p style={{ color: "#64748b" }}>

        Edit <code>backend/config/default.toml</code> <code>[ai]</code> and <code>[privacy]</code>{" "}

        sections and restart the backend to change provider or model.

      </p>

      <p>

        <a href="/docs/user-guides/US-0008.md" target="_blank" rel="noreferrer">

          Operator guide: US-0008 (local AI providers)

        </a>

      </p>

      {ai && (

        <table>

          <thead>

            <tr>

              <th>Provider</th>

              <th>Model</th>

              <th>Base URL</th>

              <th>Status</th>

              <th>Test</th>

            </tr>

          </thead>

          <tbody>

            <tr>

              <td>

                {ai.provider_label}

                {ai.is_local && (

                  <span

                    className="badge"

                    style={{

                      marginLeft: "0.5rem",

                      background: "#e0e7ff",

                      color: "#3730a3",

                    }}

                  >

                    Local

                  </span>

                )}

              </td>

              <td>{ai.model}</td>

              <td style={{ fontSize: "0.85rem", wordBreak: "break-all" }}>

                {ai.base_url || "—"}

              </td>

              <td>

                {providerConfigured ? (

                  <span className="badge" style={{ background: "#dcfce7", color: "#166534" }}>

                    Configured

                  </span>

                ) : (

                  <span className="badge" style={{ background: "#fef3c7", color: "#92400e" }}>

                    Not configured

                  </span>

                )}

              </td>

              <td>

                <button

                  type="button"

                  className="btn"

                  disabled={!providerConfigured || aiTestMutation.isPending}

                  onClick={() => aiTestMutation.mutate()}

                >

                  Test AI provider

                </button>

              </td>

            </tr>

          </tbody>

        </table>

      )}

      {aiTestMutation.data && (

        <p style={{ marginTop: "0.5rem" }}>

          {aiTestMutation.data.ok ? (

            <>

              Test OK — {aiTestMutation.data.latency_ms ?? "?"} ms

              {aiTestMutation.data.sample

                ? ` — sample: "${aiTestMutation.data.sample}"`

                : ""}

            </>

          ) : (

            <span style={{ color: "#b91c1c" }}>

              Test failed: {aiTestMutation.data.error ?? "unknown error"}

            </span>

          )}

        </p>

      )}

      {s?.privacy && (

        <table style={{ marginTop: "1rem" }}>

          <tbody>

            <tr>

              <th>Allow raw transactions</th>

              <td>{s.privacy.allow_raw_transactions ? "yes" : "no"}</td>

            </tr>

            <tr>

              <th>Redact IBAN</th>

              <td>{s.privacy.redact_iban ? "yes" : "no"}</td>

            </tr>

            <tr>

              <th>Redact counterparties</th>

              <td>{s.privacy.redact_counterparties ? "yes" : "no"}</td>

            </tr>

          </tbody>

        </table>

      )}



      <h2 style={{ marginTop: "2rem" }}>Crypto exchanges</h2>

      <p style={{ color: "#64748b" }}>

        Read-only display. Edit <code>backend/config/default.toml</code> <code>[exchanges.*]</code>{" "}

        and set env vars (<code>BINANCE_*</code>, <code>BYBIT_*</code>, <code>BITUNIX_*</code>) then{" "}

        <strong>restart the backend</strong>. Secrets are never stored in the browser.

      </p>

      <p>

        <a href="/docs/user-guides/US-0007.md" target="_blank" rel="noreferrer">

          Operator guide: US-0007

        </a>

      </p>

      {exchanges && (

        <table>

          <thead>

            <tr>

              <th>Exchange</th>

              <th>Enabled</th>

              <th>Configured</th>

              <th>API key env</th>

              <th>Test</th>

            </tr>

          </thead>

          <tbody>

            {exchangeRows.map((row) => (

              <tr key={row.id}>

                <td>{row.label}</td>

                <td>{row.cfg.enabled ? "yes" : "no"}</td>

                <td>

                  {row.cfg.configured ? (

                    <span className="badge" style={{ background: "#dcfce7", color: "#166534" }}>

                      Configured

                    </span>

                  ) : (

                    <span className="badge" style={{ background: "#fef3c7", color: "#92400e" }}>

                      Not configured

                    </span>

                  )}

                </td>

                <td>{row.cfg.api_key_env}</td>

                <td>

                  <button

                    type="button"

                    className="btn"

                    disabled={!row.cfg.enabled || exchangeTestMutation.isPending}

                    onClick={() => exchangeTestMutation.mutate(row.id)}

                  >

                    Test connection

                  </button>

                </td>

              </tr>

            ))}

          </tbody>

        </table>

      )}

      {exchangeTestMutation.data && (

        <p style={{ marginTop: "0.5rem" }}>

          Last exchange test: {exchangeTestMutation.data.ok ? "OK" : "Failed"} —{" "}

          {exchangeTestMutation.data.message} ({exchangeTestMutation.data.latency_ms} ms)

        </p>

      )}



      <h2 style={{ marginTop: "2rem" }}>Tool audit log</h2>

      <p style={{ color: "#64748b", fontSize: "0.9rem" }}>

        Redacted tool arguments only — no prompts, responses, API keys, or raw transaction rows.

        Retention: 500 rows / 90 days (startup purge; configure in TOML).

      </p>

      <table>

        <thead>

          <tr>

            <th>Time</th>

            <th>Tool</th>

            <th>Provider</th>

            <th>User</th>

            <th>Duration (ms)</th>

            <th>Status</th>

          </tr>

        </thead>

        <tbody>

          {(auditQuery.data ?? []).map((row) => (

            <tr key={row.id}>

              <td>{new Date(row.created_at).toLocaleString()}</td>

              <td>{row.tool_name}</td>

              <td>{row.provider ?? "—"}</td>

              <td>{row.user_subject}</td>

              <td>{row.duration_ms}</td>

              <td>{row.result_status}</td>

            </tr>

          ))}

        </tbody>

      </table>

      <div style={{ display: "flex", gap: "0.5rem", marginTop: "0.75rem" }}>

        <button

          type="button"

          className="btn"

          disabled={auditOffset === 0}

          onClick={() => setAuditOffset((o) => Math.max(0, o - limit))}

        >

          Previous

        </button>

        <button

          type="button"

          className="btn"

          disabled={(auditQuery.data?.length ?? 0) < limit}

          onClick={() => setAuditOffset((o) => o + limit)}

        >

          Next

        </button>

      </div>

    </div>

  );

}

