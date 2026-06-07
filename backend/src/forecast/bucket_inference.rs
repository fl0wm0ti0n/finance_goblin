use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::warn;

use super::categories::Bucket;
use super::types::RecurringPattern;
use crate::ai::privacy::{BucketFeatureRow, PrivacyLayer, RawBucketFeatureInput};
use crate::ai::provider::{
    AiProvider, ChatCompletionMessage, ChatCompletionRequest, ChatCompletionResponse, ProviderError,
};
use crate::config::ForecastConfig;

pub const BATCH_CAP: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BucketSource {
    Config,
    Ai,
    Default,
}

impl BucketSource {
    pub fn as_str(self) -> &'static str {
        match self {
            BucketSource::Config => "config",
            BucketSource::Ai => "ai",
            BucketSource::Default => "default",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BucketAssignment {
    pub feature_id: String,
    pub bucket: Bucket,
    pub confidence: f64,
    pub source: BucketSource,
    pub rationale_code: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BucketAuditEvent {
    pub feature_id: String,
    pub bucket: String,
    pub confidence: f64,
    pub source: String,
    pub rationale_code: String,
    pub result_status: String,
}

pub struct BucketInferenceService {
    provider: Option<Arc<dyn AiProvider>>,
    privacy: PrivacyLayer,
    min_confidence: f64,
    category_buckets_hash: u64,
    memo: HashMap<String, BucketAssignment>,
    audit_events: Vec<BucketAuditEvent>,
}

impl BucketInferenceService {
    pub fn new(
        provider: Option<Arc<dyn AiProvider>>,
        privacy: PrivacyLayer,
        min_confidence: f64,
        category_buckets: &HashMap<String, String>,
    ) -> Self {
        Self {
            provider,
            privacy,
            min_confidence,
            category_buckets_hash: hash_category_buckets(category_buckets),
            memo: HashMap::new(),
            audit_events: Vec::new(),
        }
    }

    pub fn category_buckets_hash(&self) -> u64 {
        self.category_buckets_hash
    }

    pub fn take_audit_events(&mut self) -> Vec<BucketAuditEvent> {
        std::mem::take(&mut self.audit_events)
    }

    pub fn is_provider_available(&self) -> bool {
        self.provider
            .as_ref()
            .is_some_and(|p| p.is_configured())
    }

    pub async fn infer_batch(
        &mut self,
        inputs: Vec<RawBucketFeatureInput>,
    ) -> HashMap<String, BucketAssignment> {
        let mut assignments = HashMap::new();
        if inputs.is_empty() {
            return assignments;
        }

        let mut pending_llm: Vec<BucketFeatureRow> = Vec::new();
        let prepared = self.privacy.prepare_bucket_features(&inputs);

        for row in prepared {
            if let Some(memo_hit) = self.memo.get(&row.feature_id).cloned() {
                assignments.insert(row.feature_id.clone(), memo_hit);
                continue;
            }

            if let Some(rule) = apply_rule_heuristic(&row) {
                self.record_assignment(&mut assignments, rule);
                continue;
            }

            pending_llm.push(row);
        }

        if pending_llm.is_empty() {
            return assignments;
        }

        let provider = match &self.provider {
            Some(p) if p.is_configured() => Arc::clone(p),
            _ => {
                for row in pending_llm {
                    let fallback = variable_fallback(&row.feature_id, "provider_unavailable");
                    self.record_assignment(&mut assignments, fallback);
                }
                return assignments;
            }
        };

        for chunk in pending_llm.chunks(BATCH_CAP) {
            match infer_llm_batch(provider.as_ref(), chunk, self.min_confidence).await {
                Ok(batch) => {
                    for assignment in batch {
                        self.record_assignment(&mut assignments, assignment);
                    }
                }
                Err(e) => {
                    warn!(?e, "bucket inference LLM batch failed");
                    for row in chunk {
                        let fallback = variable_fallback(&row.feature_id, "provider_unavailable");
                        self.record_assignment(&mut assignments, fallback);
                    }
                }
            }
        }

        assignments
    }

    fn record_assignment(
        &mut self,
        map: &mut HashMap<String, BucketAssignment>,
        assignment: BucketAssignment,
    ) {
        self.push_audit(&assignment);
        self.memo
            .insert(assignment.feature_id.clone(), assignment.clone());
        map.insert(assignment.feature_id.clone(), assignment);
    }

    fn push_audit(&mut self, assignment: &BucketAssignment) {
        let result_status = if assignment.rationale_code == "provider_unavailable" {
            "provider_unavailable"
        } else if assignment.rationale_code == "low_confidence" {
            "low_confidence"
        } else if assignment.rationale_code == "parse_error" {
            "parse_error"
        } else {
            "ok"
        };
        self.audit_events.push(BucketAuditEvent {
            feature_id: assignment.feature_id.clone(),
            bucket: assignment.bucket.as_str().to_string(),
            confidence: assignment.confidence,
            source: assignment.source.as_str().to_string(),
            rationale_code: assignment.rationale_code.clone(),
            result_status: result_status.to_string(),
        });
    }
}

pub fn feature_id_for_pattern(pattern: &RecurringPattern) -> String {
    let sign = amount_sign(pattern.amount);
    format!(
        "{}:{}:{}",
        pattern.description,
        pattern.category_id.as_deref().unwrap_or(""),
        sign
    )
}

pub fn amount_sign(amount: f64) -> i8 {
    if amount > 0.0 {
        1
    } else if amount < 0.0 {
        -1
    } else {
        0
    }
}

pub fn collect_ambiguous_features(
    patterns: &[RecurringPattern],
    category_names: &HashMap<String, String>,
    config: &ForecastConfig,
) -> Vec<RawBucketFeatureInput> {
    patterns
        .iter()
        .filter(|p| is_ai_eligible(p.category_id.as_deref(), category_names, config))
        .map(|p| {
            let category_name = p
                .category_id
                .as_deref()
                .and_then(|id| category_names.get(id))
                .map(|s| s.to_string());
            RawBucketFeatureInput {
                feature_id: feature_id_for_pattern(p),
                category_name,
                payee_normalized: p.description.clone(),
                amount: p.amount,
                recurring_label: None,
                pattern_class: classify_pattern(p),
            }
        })
        .collect()
}

pub fn is_ai_eligible(
    category_id: Option<&str>,
    category_names: &HashMap<String, String>,
    config: &ForecastConfig,
) -> bool {
    let name = category_id
        .and_then(|id| category_names.get(id))
        .map(|s| s.trim().to_lowercase())
        .unwrap_or_default();
    if name.is_empty() {
        return true;
    }
    !config.category_buckets.contains_key(&name)
}

fn classify_pattern(pattern: &RecurringPattern) -> String {
    if pattern.amount > 0.0 {
        return "discretionary".into();
    }
    if pattern.interval_days >= 28 && pattern.interval_days <= 31 && pattern.amount <= -200.0 {
        "standing_order".into()
    } else if pattern.interval_days >= 28 && pattern.interval_days <= 31 {
        "subscription".into()
    } else {
        "discretionary".into()
    }
}

fn apply_rule_heuristic(row: &BucketFeatureRow) -> Option<BucketAssignment> {
    if row.amount_sign > 0 {
        return Some(BucketAssignment {
            feature_id: row.feature_id.clone(),
            bucket: Bucket::Income,
            confidence: 0.99,
            source: BucketSource::Default,
            rationale_code: "rule_inflow".into(),
        });
    }
    if row.pattern_class == "standing_order" && row.amount_sign < 0 {
        return Some(BucketAssignment {
            feature_id: row.feature_id.clone(),
            bucket: Bucket::Fixed,
            confidence: 0.99,
            source: BucketSource::Default,
            rationale_code: "rule_standing_order".into(),
        });
    }
    if row.amount_sign < 0 && row.magnitude_band == "200+" {
        return Some(BucketAssignment {
            feature_id: row.feature_id.clone(),
            bucket: Bucket::Fixed,
            confidence: 0.98,
            source: BucketSource::Default,
            rationale_code: "rule_large_outflow".into(),
        });
    }
    None
}

fn variable_fallback(feature_id: &str, rationale: &str) -> BucketAssignment {
    BucketAssignment {
        feature_id: feature_id.to_string(),
        bucket: Bucket::Variable,
        confidence: 0.0,
        source: BucketSource::Default,
        rationale_code: rationale.to_string(),
    }
}

fn hash_category_buckets(map: &HashMap<String, String>) -> u64 {
    let mut keys: Vec<_> = map.keys().collect();
    keys.sort();
    let mut hasher = DefaultHasher::new();
    for k in keys {
        k.hash(&mut hasher);
        map[k].hash(&mut hasher);
    }
    hasher.finish()
}

#[derive(Debug, Deserialize)]
struct LlmBucketRow {
    feature_id: String,
    bucket: String,
    confidence: f64,
    rationale_code: String,
}

async fn infer_llm_batch(
    provider: &dyn AiProvider,
    rows: &[BucketFeatureRow],
    min_confidence: f64,
) -> Result<Vec<BucketAssignment>, ProviderError> {
    let payload: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "feature_id": r.feature_id,
                "category_name": r.category_name,
                "merchant_token": r.merchant_token,
                "amount_sign": r.amount_sign,
                "magnitude_band": r.magnitude_band,
                "recurring_label": r.recurring_label,
                "pattern_class": r.pattern_class,
            })
        })
        .collect();

    let system = "You classify recurring cashflow rows into forecast buckets: income, fixed, or variable. \
Respond with ONLY a JSON array of objects: \
{\"feature_id\":string,\"bucket\":\"income\"|\"fixed\"|\"variable\",\"confidence\":number,\"rationale_code\":string} \
One object per input feature_id. No markdown.";

    let user = format!("Classify these privacy-safe features:\n{}", serde_json::to_string(&payload).unwrap_or_default());

    let req = ChatCompletionRequest {
        model: provider.model().to_string(),
        messages: vec![
            ChatCompletionMessage {
                role: "system".into(),
                content: Some(system.into()),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
            ChatCompletionMessage {
                role: "user".into(),
                content: Some(user),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
        ],
        tools: None,
        tool_choice: None,
        temperature: Some(0.0),
        max_tokens: provider.max_completion_tokens(),
        stream: false,
    };

    let resp = provider.chat_completion(req).await?;
    let text = resp
        .choices
        .first()
        .and_then(|c| c.message.content.clone())
        .unwrap_or_default();

    parse_llm_assignments(&text, rows, min_confidence)
}

fn parse_llm_assignments(
    text: &str,
    rows: &[BucketFeatureRow],
    min_confidence: f64,
) -> Result<Vec<BucketAssignment>, ProviderError> {
    let trimmed = text.trim();
    let json_text = if let Some(start) = trimmed.find('[') {
        if let Some(end) = trimmed.rfind(']') {
            &trimmed[start..=end]
        } else {
            trimmed
        }
    } else {
        trimmed
    };

    let parsed: Vec<LlmBucketRow> = serde_json::from_str(json_text)
        .map_err(|e| ProviderError::Parse(format!("bucket LLM JSON: {e}")))?;

    let row_ids: std::collections::HashSet<_> = rows.iter().map(|r| r.feature_id.as_str()).collect();
    let mut out = Vec::new();

    for item in parsed {
        if !row_ids.contains(item.feature_id.as_str()) {
            continue;
        }
        let bucket = parse_bucket_str(&item.bucket);
        let (bucket, confidence, source, rationale) = if item.confidence >= min_confidence {
            (bucket, item.confidence, BucketSource::Ai, item.rationale_code)
        } else {
            (
                Bucket::Variable,
                item.confidence,
                BucketSource::Default,
                "low_confidence".to_string(),
            )
        };
        out.push(BucketAssignment {
            feature_id: item.feature_id,
            bucket,
            confidence,
            source,
            rationale_code: rationale,
        });
    }

    for row in rows {
        if !out.iter().any(|a| a.feature_id == row.feature_id) {
            out.push(variable_fallback(&row.feature_id, "parse_error"));
        }
    }

    Ok(out)
}

fn parse_bucket_str(s: &str) -> Bucket {
    match s.trim().to_lowercase().as_str() {
        "income" => Bucket::Income,
        "fixed" => Bucket::Fixed,
        _ => Bucket::Variable,
    }
}

use serde_json::Value;

/// Test-only mock provider for bucket inference unit tests.
#[cfg(test)]
pub struct MockBucketProvider {
    pub response: String,
    pub should_fail: bool,
}

#[cfg(test)]
#[async_trait]
impl AiProvider for MockBucketProvider {
    fn name(&self) -> &str {
        "mock"
    }
    fn is_configured(&self) -> bool {
        true
    }
    fn is_local(&self) -> bool {
        true
    }
    fn display_label(&self) -> &str {
        "Mock"
    }
    fn omit_tool_choice(&self) -> bool {
        true
    }
    fn default_temperature(&self) -> f32 {
        0.0
    }
    fn model(&self) -> &str {
        "mock"
    }
    fn max_completion_tokens(&self) -> u32 {
        512
    }
    fn base_url(&self) -> &str {
        "mock://"
    }
    async fn chat_completion(
        &self,
        _req: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, ProviderError> {
        if self.should_fail {
            return Err(ProviderError::Request("mock failure".into()));
        }
        Ok(ChatCompletionResponse {
            choices: vec![crate::ai::provider::ChatChoice {
                message: ChatCompletionMessage {
                    role: "assistant".into(),
                    content: Some(self.response.clone()),
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                },
                finish_reason: Some("stop".into()),
            }],
        })
    }
    async fn chat_completion_stream(
        &self,
        _req: ChatCompletionRequest,
    ) -> Result<reqwest::Response, ProviderError> {
        Err(ProviderError::Request("not implemented".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::privacy::PrivacyLayer;
    use crate::config::PrivacyConfig;
    use crate::forecast::categories::default_category_buckets;

    fn svc_with_mock(response: &str, min_conf: f64) -> BucketInferenceService {
        let privacy = PrivacyLayer::new(PrivacyConfig {
            allow_raw_transactions: false,
            redact_iban: true,
            redact_counterparties: true,
        });
        let provider: Arc<dyn AiProvider> = Arc::new(MockBucketProvider {
            response: response.into(),
            should_fail: false,
        });
        BucketInferenceService::new(
            Some(provider),
            privacy,
            min_conf,
            &default_category_buckets(),
        )
    }

    #[tokio::test]
    async fn threshold_boundary_074_falls_back_to_variable() {
        let json = r#"[{"feature_id":"x:::-1","bucket":"fixed","confidence":0.74,"rationale_code":"llm"}]"#;
        let mut svc = svc_with_mock(json, 0.75);
        let inputs = vec![RawBucketFeatureInput {
            feature_id: "x:::-1".into(),
            category_name: None,
            payee_normalized: "merchant".into(),
            amount: -50.0,
            recurring_label: None,
            pattern_class: "discretionary".into(),
        }];
        let map = svc.infer_batch(inputs).await;
        let a = map.get("x:::-1").unwrap();
        assert_eq!(a.bucket, Bucket::Variable);
        assert_eq!(a.rationale_code, "low_confidence");
    }

    #[tokio::test]
    async fn threshold_boundary_075_applies_ai_bucket() {
        let json = r#"[{"feature_id":"x:::-1","bucket":"fixed","confidence":0.75,"rationale_code":"llm"}]"#;
        let mut svc = svc_with_mock(json, 0.75);
        let inputs = vec![RawBucketFeatureInput {
            feature_id: "x:::-1".into(),
            category_name: None,
            payee_normalized: "merchant".into(),
            amount: -50.0,
            recurring_label: None,
            pattern_class: "discretionary".into(),
        }];
        let map = svc.infer_batch(inputs).await;
        let a = map.get("x:::-1").unwrap();
        assert_eq!(a.bucket, Bucket::Fixed);
        assert_eq!(a.source, BucketSource::Ai);
    }

    #[tokio::test]
    async fn provider_unavailable_falls_back() {
        let privacy = PrivacyLayer::new(PrivacyConfig {
            allow_raw_transactions: false,
            redact_iban: true,
            redact_counterparties: true,
        });
        let mut svc = BucketInferenceService::new(None, privacy, 0.75, &default_category_buckets());
        let inputs = vec![RawBucketFeatureInput {
            feature_id: "y:::-1".into(),
            category_name: None,
            payee_normalized: "small spend".into(),
            amount: -10.0,
            recurring_label: None,
            pattern_class: "discretionary".into(),
        }];
        let map = svc.infer_batch(inputs).await;
        let a = map.get("y:::-1").unwrap();
        assert_eq!(a.bucket, Bucket::Variable);
        assert_eq!(a.rationale_code, "provider_unavailable");
    }

    #[tokio::test]
    async fn batch_cap_splits_at_100() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let privacy = PrivacyLayer::new(PrivacyConfig {
            allow_raw_transactions: false,
            redact_iban: true,
            redact_counterparties: true,
        });
        let call_count = Arc::new(AtomicUsize::new(0));
        struct CountingProvider {
            calls: Arc<AtomicUsize>,
        }
        #[async_trait]
        impl AiProvider for CountingProvider {
            fn name(&self) -> &str {
                "count"
            }
            fn is_configured(&self) -> bool {
                true
            }
            fn is_local(&self) -> bool {
                true
            }
            fn display_label(&self) -> &str {
                "Count"
            }
            fn omit_tool_choice(&self) -> bool {
                true
            }
            fn default_temperature(&self) -> f32 {
                0.0
            }
            fn model(&self) -> &str {
                "mock"
            }
            fn max_completion_tokens(&self) -> u32 {
                512
            }
            fn base_url(&self) -> &str {
                "mock://"
            }
            async fn chat_completion(
                &self,
                _req: ChatCompletionRequest,
            ) -> Result<ChatCompletionResponse, ProviderError> {
                self.calls.fetch_add(1, Ordering::SeqCst);
                Ok(ChatCompletionResponse {
                    choices: vec![crate::ai::provider::ChatChoice {
                        message: ChatCompletionMessage {
                            role: "assistant".into(),
                            content: Some("[]".into()),
                            tool_calls: None,
                            tool_call_id: None,
                            name: None,
                        },
                        finish_reason: Some("stop".into()),
                    }],
                })
            }
            async fn chat_completion_stream(
                &self,
                _req: ChatCompletionRequest,
            ) -> Result<reqwest::Response, ProviderError> {
                Err(ProviderError::Request("n/a".into()))
            }
        }
        let provider: Arc<dyn AiProvider> = Arc::new(CountingProvider {
            calls: Arc::clone(&call_count),
        });
        let mut svc = BucketInferenceService::new(
            Some(provider),
            privacy,
            0.75,
            &default_category_buckets(),
        );
        let inputs: Vec<_> = (0..101)
            .map(|i| RawBucketFeatureInput {
                feature_id: format!("f{i}:::-1"),
                category_name: None,
                payee_normalized: format!("payee-{i}"),
                amount: -5.0,
                recurring_label: None,
                pattern_class: "discretionary".into(),
            })
            .collect();
        let _ = svc.infer_batch(inputs).await;
        assert_eq!(
            call_count.load(Ordering::SeqCst),
            2,
            "101 rows should produce 2 LLM batches"
        );
    }

    #[test]
    fn rule_inflow_short_circuits_before_llm() {
        let row = BucketFeatureRow {
            feature_id: "z:::1".into(),
            category_name: None,
            merchant_token: "Counterparty-abc".into(),
            amount_sign: 1,
            magnitude_band: "200+".into(),
            recurring_label: None,
            pattern_class: "discretionary".into(),
        };
        let rule = apply_rule_heuristic(&row).unwrap();
        assert_eq!(rule.bucket, Bucket::Income);
        assert!(rule.confidence >= 0.98);
    }
}
