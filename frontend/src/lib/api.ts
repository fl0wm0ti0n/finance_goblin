type TokenProvider = () => string | null;

let accessTokenProvider: TokenProvider = () => null;

export function setAccessTokenProvider(provider: TokenProvider) {
  accessTokenProvider = provider;
}

const apiBase = import.meta.env.VITE_API_BASE_URL ?? "";

export async function apiFetch<T>(path: string, init: RequestInit = {}): Promise<T> {
  const token = accessTokenProvider();
  const headers = new Headers(init.headers);
  headers.set("Accept", "application/json");
  if (token) {
    headers.set("Authorization", `Bearer ${token}`);
  }
  if (init.body && !headers.has("Content-Type")) {
    headers.set("Content-Type", "application/json");
  }

  const resp = await fetch(`${apiBase}${path}`, {
    ...init,
    headers,
    // Required for Traefik HTTP basic-auth on same-origin API polling
    credentials: "include",
  });
  if (resp.status === 401) {
    throw new Error("unauthorized");
  }
  if (!resp.ok) {
    const text = await resp.text();
    throw new Error(text || resp.statusText);
  }
  if (resp.status === 204) {
    return undefined as T;
  }
  return resp.json() as Promise<T>;
}

export interface SyncStatus {
  state: string;
  phase: string | null;
  active_run_id: string | null;
  last_run: SyncRun | null;
}

export interface SyncRun {
  id: string;
  started_at: string;
  finished_at: string | null;
  status: string;
  trigger: string;
  error_message: string | null;
}

export interface EntityCounts {
  entities?: { entity: string; count: number }[];
  accounts?: number;
  transactions?: number;
  categories?: number;
  budgets?: number;
  tags?: number;
  piggy_banks?: number;
}

export interface AiPublicSettings {
  provider: string;
  provider_label: string;
  base_url: string;
  model: string;
  is_local: boolean;
  provider_configured: boolean;
}

export interface AiTestResponse {
  ok: boolean;
  latency_ms?: number;
  model?: string;
  provider: string;
  sample?: string;
  error?: string;
}

export interface PrivacySettings {
  allow_raw_transactions: boolean;
  redact_iban: boolean;
  redact_counterparties: boolean;
}

export interface ExchangeSettingsView {
  enabled: boolean;
  api_key_env: string;
  api_secret_env: string;
  configured: boolean;
  base_url?: string | null;
  spot_base_url?: string | null;
  enabled_futures?: boolean | null;
}

export interface ExchangesSettingsView {
  enabled: boolean;
  interval_seconds: number;
  binance: ExchangeSettingsView;
  bybit: ExchangeSettingsView;
  bitunix: ExchangeSettingsView;
}

export interface PortfolioSettingsView {
  trade_retention_days: number;
  frankfurter_base_url: string;
}

export interface Settings {
  firefly_base_url: string;
  firefly_auth_method: string;
  database_mode: string;
  sync_interval_seconds: number;
  oidc_issuer_url: string;
  read_only: boolean;
  ai?: AiPublicSettings;
  privacy?: PrivacySettings;
  provider_configured?: boolean;
  openai_configured?: boolean;
  exchanges?: ExchangesSettingsView;
  portfolio?: PortfolioSettingsView;
}

export interface AiAuditRow {
  id: string;
  session_id: string;
  user_subject: string;
  tool_name: string;
  args_summary: Record<string, unknown>;
  result_status: string;
  duration_ms: number;
  error_message?: string | null;
  model?: string | null;
  provider?: string | null;
  created_at: string;
}

export interface TriggerResponse {
  run_id: string;
  status: string;
}

export interface SubscriptionTag {
  id: string;
  name: string;
  slug: string;
}

export interface SubscriptionPattern {
  id: string;
  fingerprint: string;
  status: string;
  kind: string;
  payee_key: string;
  display_name: string;
  interval_days: number;
  current_amount: string;
  confidence_pct: number;
  first_seen_at: string;
  last_seen_at: string;
  confirmed_at?: string | null;
  rejected_at?: string | null;
  display_category_id?: string | null;
  transaction_count?: number;
  tags?: SubscriptionTag[];
}

export interface DiscoverCandidate {
  payee_key: string;
  display_name: string;
  interval_days: number;
  median_amount: number;
  confidence_pct: number;
  transaction_count: number;
  transaction_ids: string[];
  account_ids: string[];
}

export interface DiscoverResponse {
  candidates: DiscoverCandidate[];
  meta: { limit: number; truncated: boolean; window_days: number };
}

export interface DiscoverConfirmResponse {
  pattern: SubscriptionPattern;
  merged: boolean;
}

export interface OperatorTag {
  id: string;
  name: string;
  slug: string;
  created_at: string;
  updated_at: string;
}

export function fetchDiscover(params: {
  account_id: string;
  payee?: string;
  interval_days?: number;
  limit?: number;
}): Promise<DiscoverResponse> {
  const qs = new URLSearchParams({ account_id: params.account_id });
  if (params.payee) qs.set("payee", params.payee);
  if (params.interval_days != null) qs.set("interval_days", String(params.interval_days));
  if (params.limit != null) qs.set("limit", String(params.limit));
  return apiFetch<DiscoverResponse>(`/api/v1/subscriptions/discover?${qs}`);
}

export function confirmDiscoverCandidate(body: {
  payee_key: string;
  interval_days: number;
  median_amount: number;
  transaction_ids: string[];
  kind?: string;
}): Promise<DiscoverConfirmResponse> {
  return apiFetch<DiscoverConfirmResponse>("/api/v1/subscriptions/discover/confirm", {
    method: "POST",
    body: JSON.stringify(body),
  });
}

export function fetchOperatorTags(): Promise<OperatorTag[]> {
  return apiFetch<OperatorTag[]>("/api/v1/subscription-tags");
}

export function createOperatorTag(name: string): Promise<OperatorTag> {
  return apiFetch<OperatorTag>("/api/v1/subscription-tags", {
    method: "POST",
    body: JSON.stringify({ name }),
  });
}

export function renameOperatorTag(id: string, name: string): Promise<OperatorTag> {
  return apiFetch<OperatorTag>(`/api/v1/subscription-tags/${id}`, {
    method: "PATCH",
    body: JSON.stringify({ name }),
  });
}

export function deleteOperatorTag(id: string): Promise<void> {
  return apiFetch<void>(`/api/v1/subscription-tags/${id}`, { method: "DELETE" });
}

export function assignSubscriptionTags(id: string, tagIds: string[]): Promise<SubscriptionTag[]> {
  return apiFetch<SubscriptionTag[]>(`/api/v1/subscriptions/${id}/tags`, {
    method: "PUT",
    body: JSON.stringify({ tag_ids: tagIds }),
  });
}

export interface SubscriptionAlert {
  id: string;
  pattern_id: string | null;
  alert_type: string;
  title: string;
  body: string | null;
  read_at: string | null;
  created_at: string;
}

export interface SubscriptionUnreadCount {
  unread_total: number;
  unread_new_detection: number;
  unread_price_change: number;
  pending_patterns: number;
  reconciled: boolean;
  reconciliation_note: string;
}

export interface ForecastBalanceWarning {
  account_id: string;
  starting_balance: number;
  reason: string;
}

export interface ForecastMeta {
  computation_id: string | null;
  computed_at: string | null;
  stale: boolean;
  low_confidence: boolean;
  sync_run_id: string | null;
  baseline_computation_id?: string | null;
  ml_computation_id?: string | null;
  ml_status?: string | null;
  ml_skipped_reason?: string | null;
  balance_warnings?: ForecastBalanceWarning[] | null;
  seasonal_detected?: boolean | null;
  seasonal_periods?: number[] | null;
  seasonal_strength?: number | null;
  model_family?: string | null;
  backtest_wmape?: number | null;
}

export interface ForecastLongTermPoint {
  date: string;
  balance: string;
  balance_p10?: string;
  balance_p90?: string;
}

export interface ForecastLongTerm {
  variant?: string;
  series: ForecastLongTermPoint[];
  end_balance: string;
  end_balance_p10?: string | null;
  end_balance_p90?: string | null;
  model_family?: string | null;
  seasonal_periods?: number[] | null;
  backtest_wmape?: number | null;
  low_confidence?: boolean | null;
}

export interface ForecastCompare {
  horizon_months: number;
  baseline: {
    end_balance: string;
    series: ForecastLongTermPoint[];
  };
  ml_enhanced?: {
    end_balance: string;
    end_balance_p10?: string;
    end_balance_p90?: string;
    series: ForecastLongTermPoint[];
  } | null;
  delta_end_balance?: string | null;
  ml_available: boolean;
  ml_skipped_reason?: string | null;
}

export type PlanRiskScoreResponse =
  | {
      status: "ok";
      score: number;
      band: string;
      components: {
        balance_stress: number;
        plan_viability: number;
        crypto_volatility: number;
        ml_divergence_modifier: number;
      };
      plan_computation_id: string;
    }
  | {
      status: "no_score";
      reason: "no_active_plan" | "not_computed";
    };

export interface PortfolioForecast {
  horizons: {
    months: number;
    value_eur: string;
    value_p10?: string | null;
    value_p90?: string | null;
  }[];
  low_confidence: boolean;
  fx_incomplete_warning: boolean;
  skipped: boolean;
  skip_reason?: string | null;
}

export interface ForecastAccount {
  id: string;
  name: string;
  currency: string | null;
}

export interface ForecastDaily {
  milestones: {
    tomorrow: string;
    next_week: string;
    month_end: string;
  };
  series: { date: string; balance: string }[];
}

export interface ForecastMonthly {
  series: {
    month: string;
    income: string;
    fixed_costs: string;
    variable_costs: string;
    free_cashflow: string;
    bucket_sources?: {
      income: string;
      fixed_costs: string;
      variable_costs: string;
    };
    ai_mapped?: boolean;
  }[];
  seasonal?: {
    seasonal_detected: boolean;
    seasonal_periods: number[];
    seasonal_strength?: number | null;
  };
}

export interface PlanListItem {
  id: string;
  name: string;
  template: string;
  is_active: boolean;
  latest_version_id: string | null;
  latest_version_number: number | null;
  plan_stale: boolean;
}

export interface PlanDetail {
  plan: { id: string; name: string; template: string; is_active: boolean };
  versions: { id: string; version_number: number; is_latest: boolean; frozen: boolean }[];
}

export interface PlanAdjustment {
  id: string;
  direction: string;
  amount: string;
  frequency: string;
  target_type: string;
  target_key?: string | null;
  label?: string | null;
  effective_from: string;
  effective_to?: string | null;
  sort_order: number;
}

export interface PlanCompare {
  plan_id: string;
  plan_name: string;
  versions: {
    version_id: string;
    version_number: number;
    frozen: boolean;
    monthly_delta_sum: string;
    projected_month_end_balance: string;
  }[];
  at_version_cap: boolean;
}

export interface PlanVsActualRow {
  date: string;
  planned: string | null;
  actual: string | null;
  deviation: string | null;
}

export type PlanVsActual =
  | {
      status: "ok";
      month: string;
      reporting_currency: string;
      plan_stale: boolean;
      actuals_stale: boolean;
      rows: PlanVsActualRow[];
    }
  | {
      status: "no_active_plan";
      reason: "no_active_plan";
    };

export interface SavingsSuggestion {
  pattern_id: string;
  payee_key: string;
  display_name: string;
  current_amount: string;
  interval_days: number;
}

export interface GoalStats {
  plan_id: string;
  version_id: string;
  target_balance_eur: string;
  target_date: string;
  goal_account_id: string | null;
  monthly_delta_vs_baseline: string;
  yearly_rollup: { year: number; planned_net_sum: string }[];
  projected_balance_at_target: string | null;
  gap_eur: string | null;
  required_monthly_savings_eur: string | null;
  on_track: boolean;
  beyond_horizon: boolean;
  computed_at: string | null;
  household_fallback: boolean;
}

export interface CategorySavingsSuggestion {
  category_id: string;
  category_name: string;
  avg_monthly_outflow_eur: string;
  transaction_count: number;
  suggested_reduction_eur: string;
  evidence_summary: string;
}

export interface CategorySavingsResponse {
  suggestions: CategorySavingsSuggestion[];
  meta: { months: number; limit: number; ranking: string };
}

export function fetchGoalStats(planId: string, versionId?: string): Promise<GoalStats> {
  const qs = versionId ? `?version_id=${encodeURIComponent(versionId)}` : "";
  return apiFetch<GoalStats>(`/api/v1/plans/${planId}/goal-stats${qs}`);
}

export function fetchCategorySavingsSuggestions(
  planId: string,
  months = 6,
  limit = 10,
): Promise<CategorySavingsResponse> {
  const params = new URLSearchParams({ months: String(months), limit: String(limit) });
  return apiFetch<CategorySavingsResponse>(
    `/api/v1/plans/${planId}/category-savings-suggestions?${params}`,
  );
}

export interface AccountWealthRow {
  firefly_id: string;
  name: string;
  account_role?: string | null;
  currency: string;
  balance: number;
  is_overdrawn?: boolean;
  pct_of_total?: number | null;
}

export interface CryptoHoldingRow {
  exchange_id: string;
  asset: string;
  quantity: number;
  value_eur: number;
  unrealized_pnl_eur?: number | null;
  product_type: string;
}

export interface HoldingsAllRow {
  asset: string;
  quantity: number;
  product_type: string;
  value_eur: number | null;
  unrealized_pnl_eur?: number | null;
  native_unit: string;
}

export type AdjustmentTargetType =
  | "household"
  | "subscription"
  | "category"
  | "custom_label"
  | "allocation_target";

export interface CryptoBreakdown {
  subtotal_eur: number;
  fx_complete: boolean;
  exchanges: {
    id: string;
    connection_state: string;
    last_sync_at: string | null;
    subtotal_eur: number;
    holdings_count: number;
  }[];
  holdings_top: CryptoHoldingRow[];
  holdings_all: HoldingsAllRow[];
  unpriced_assets: string[];
}

export interface PnlSummary {
  realized_eur: number;
  unrealized_eur: number;
  total_return_pct?: number | null;
}

export interface AllocationGap {
  current: { etf_traditional_pct: number; crypto_pct: number; cash_pct: number };
  target: { etf_traditional_pct: number; crypto_pct: number; cash_pct: number };
  gaps: { etf_traditional_pct: number; crypto_pct: number; cash_pct: number };
}

export interface ExtendedWealthBreakdown {
  reporting_currency: string;
  firefly: {
    subtotal_eur: number;
    mixed_currency: boolean;
    accounts: AccountWealthRow[];
  };
  crypto: CryptoBreakdown;
  total_eur: number;
  pnl: PnlSummary;
  fx_incomplete: boolean;
  crypto_placeholder: boolean;
  allocation_gap?: AllocationGap | null;
  last_successful_sync_at: string | null;
  computed_at: string;
}

export interface NetWorthBreakdown extends ExtendedWealthBreakdown {
  total: number;
  mixed_currency: boolean;
  accounts: AccountWealthRow[];
  crypto_placeholder: boolean;
}

export interface ExchangeListItem {
  id: string;
  enabled: boolean;
  connection_state: string;
  last_sync_at: string | null;
  last_error: string | null;
  counts: { holdings: number; trades: number; transfers: number; funding: number };
}

export interface WealthHistoryPoint {
  snapshot_date: string;
  total_eur: number;
  mixed_currency: boolean;
  account_count: number;
  crypto_value_eur?: number;
  firefly_value_eur?: number;
  total_return_pct?: number | null;
}

export interface AlertRow {
  id: string;
  alert_type: string;
  severity: string;
  status: string;
  fingerprint: string;
  title: string;
  message: string;
  entity_type?: string | null;
  entity_id?: string | null;
  context: Record<string, unknown>;
  triggered_at: string;
  acknowledged_at?: string | null;
  dismissed_at?: string | null;
  resolved_at?: string | null;
  stale: boolean;
}

export const UNCATEGORIZED_CATEGORY_ID = "__uncategorized__";

export interface CategoryCatalogItem {
  id: string;
  name: string;
}

export interface CategoryCatalogResponse {
  categories: CategoryCatalogItem[];
  truncated?: boolean;
}

export interface ExpenseSeriesMonth {
  month: string;
  outflow_eur: number;
  inflow_eur: number;
  transaction_count: number;
}

export interface ExpenseSeriesSummary {
  mom_delta_pct: number;
  best_month: string;
  worst_month: string;
}

export interface ExpenseSeriesResponse {
  category_id: string;
  category_name?: string;
  category_label?: string;
  uncategorized?: boolean;
  months: ExpenseSeriesMonth[];
  summary: ExpenseSeriesSummary;
  meta: { period_start: string; period_end: string };
  transaction_count: number;
}

export function fetchCategories(search?: string): Promise<CategoryCatalogResponse> {
  const q = search?.trim();
  const suffix = q && q.length >= 2 ? `?q=${encodeURIComponent(q)}` : "";
  return apiFetch<CategoryCatalogResponse>(`/api/v1/categories${suffix}`);
}

export function fetchCategoryExpenseSeries(
  categoryId: string,
  months = 12,
  end?: string,
): Promise<ExpenseSeriesResponse> {
  const params = new URLSearchParams({
    category_id: categoryId,
    months: String(months),
  });
  if (end) params.set("end", end);
  return apiFetch<ExpenseSeriesResponse>(`/api/v1/categories/expense-series?${params}`);
}

export function entityCountEntries(counts: EntityCounts | undefined): { entity: string; count: number }[] {
  if (!counts) return [];
  if ("entities" in counts && Array.isArray((counts as { entities?: { entity: string; count: number }[] }).entities)) {
    return (counts as { entities: { entity: string; count: number }[] }).entities;
  }
  return [
    { entity: "accounts", count: counts.accounts ?? 0 },
    { entity: "transactions", count: counts.transactions ?? 0 },
    { entity: "categories", count: counts.categories ?? 0 },
    { entity: "budgets", count: counts.budgets ?? 0 },
    { entity: "tags", count: counts.tags ?? 0 },
    { entity: "piggy_banks", count: counts.piggy_banks ?? 0 },
  ];
}
