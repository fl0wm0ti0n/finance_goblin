ALTER TABLE ai_tool_audit ADD COLUMN IF NOT EXISTS provider TEXT;
CREATE INDEX IF NOT EXISTS ai_tool_audit_provider
  ON ai_tool_audit (provider, created_at DESC);
