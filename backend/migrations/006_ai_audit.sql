CREATE TABLE IF NOT EXISTS ai_tool_audit (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  session_id      UUID NOT NULL,
  user_subject    TEXT NOT NULL,
  tool_name       TEXT NOT NULL CHECK (tool_name IN (
                    'get_transactions','get_subscriptions','get_forecast',
                    'get_budget_status','get_portfolio','simulate_plan')),
  args_summary    JSONB NOT NULL DEFAULT '{}',
  result_status   TEXT NOT NULL CHECK (result_status IN ('ok','error')),
  result_rows     INT,
  duration_ms     INT NOT NULL,
  error_message   TEXT,
  model           TEXT,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS ai_tool_audit_created ON ai_tool_audit (created_at DESC);
CREATE INDEX IF NOT EXISTS ai_tool_audit_tool ON ai_tool_audit (tool_name, created_at DESC);
CREATE INDEX IF NOT EXISTS ai_tool_audit_session ON ai_tool_audit (session_id, created_at DESC);
