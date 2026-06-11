-- BUG-0017 / DEC-0105: extend ai_tool_audit CHECK constraints for forecast bucket audit

ALTER TABLE ai_tool_audit DROP CONSTRAINT ai_tool_audit_tool_name_check;
ALTER TABLE ai_tool_audit ADD CONSTRAINT ai_tool_audit_tool_name_check
  CHECK (tool_name IN (
    'get_transactions','get_subscriptions','get_forecast',
    'get_budget_status','get_portfolio','simulate_plan',
    'forecast_bucket_assignment'
  )) NOT VALID;
ALTER TABLE ai_tool_audit VALIDATE CONSTRAINT ai_tool_audit_tool_name_check;

ALTER TABLE ai_tool_audit DROP CONSTRAINT ai_tool_audit_result_status_check;
ALTER TABLE ai_tool_audit ADD CONSTRAINT ai_tool_audit_result_status_check
  CHECK (result_status IN (
    'ok','error','low_confidence','provider_unavailable','parse_error'
  )) NOT VALID;
ALTER TABLE ai_tool_audit VALIDATE CONSTRAINT ai_tool_audit_result_status_check;
