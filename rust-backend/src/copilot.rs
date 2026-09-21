use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::ChainSnapshot;

const MAX_QUESTION_CHARS: usize = 1_000;

#[derive(Debug, Clone, Deserialize)]
pub struct CopilotRequest {
    pub question: String,
    pub mode: String,
    pub symbol: String,
    pub date: Option<String>,
    pub minute: Option<String>,
    pub expiration: Option<String>,
    #[serde(default = "default_pricing_mode")]
    pub pricing_mode: String,
    #[serde(default = "default_dealer_model")]
    pub dealer_model: String,
}

impl CopilotRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        let question = self.question.trim();
        if question.is_empty() {
            return Err("question must not be empty");
        }
        if question.chars().count() > MAX_QUESTION_CHARS {
            return Err("question is too long");
        }
        if !matches!(self.mode.as_str(), "replay" | "live") {
            return Err("mode must be replay or live");
        }
        if self.symbol.trim().is_empty() || self.symbol.len() > 15 {
            return Err("symbol is invalid");
        }
        Ok(())
    }
}

fn default_pricing_mode() -> String { "micro".into() }
fn default_dealer_model() -> String { "classic".into() }

#[derive(Debug, Clone, Serialize)]
pub struct EvidenceItem {
    pub id: String,
    pub label: String,
    pub value: Option<f64>,
    pub unit: String,
    pub status: &'static str,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CopilotResponse {
    pub kind: &'static str,
    pub provider: &'static str,
    pub intent: &'static str,
    pub answer: String,
    pub snapshot_id: String,
    pub as_of: String,
    pub model_version: String,
    pub evidence: Vec<EvidenceItem>,
    pub limitations: Vec<String>,
}

pub fn answer(question: &str, chain: &ChainSnapshot, volatility: &Value) -> CopilotResponse {
    let intent = classify_intent(question);
    let evidence = collect_evidence(chain, volatility);
    let answer = render_answer(intent, chain, volatility);
    let mut limitations = vec![
        "Research-only output; it is not investment advice or an order instruction.".into(),
        "V0 uses deterministic templates. A model provider is not yet enabled.".into(),
    ];
    if chain.quality.usable_pct < 70.0 {
        limitations.push("Usable quote coverage is below 70%; interpret derived metrics cautiously.".into());
    }
    if !chain.quality.gex_ready {
        limitations.push("Dealer-exposure conclusions are blocked by the current quality gate.".into());
    }

    CopilotResponse {
        kind: "copilot_response",
        provider: "deterministic-v0",
        intent,
        answer,
        snapshot_id: chain.snapshot_id.clone(),
        as_of: chain.timestamp.clone(),
        model_version: chain.provenance.model.clone(),
        evidence,
        limitations,
    }
}

fn classify_intent(question: &str) -> &'static str {
    let normalized = question.to_lowercase();
    if contains_any(&normalized, &["gamma", "gex", "flip", "伽马", "墙"]) {
        "dealer_exposure"
    } else if contains_any(&normalized, &["iv", "vol", "vrp", "波动率", "贵", "便宜", "expected move"]) {
        "volatility"
    } else if contains_any(&normalized, &["quality", "可靠", "数据", "coverage", "可信"]) {
        "data_quality"
    } else {
        "overview"
    }
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

fn collect_evidence(chain: &ChainSnapshot, volatility: &Value) -> Vec<EvidenceItem> {
    let source = format!("{} · {}", chain.provenance.source, chain.snapshot_id);
    vec![
        evidence("E1", "Spot", Some(chain.spot), "USD", &source),
        evidence("E2", "ATM IV", chain.metrics.atm_iv, "%", &source),
        evidence("E3", "RV20", value_at(volatility, &["realized_volatility", "20"]), "%", &source),
        evidence("E4", "VRP20", value_at(volatility, &["vrp20"]), "vol pts", &source),
        evidence("E5", "IV Rank", value_at(volatility, &["iv_rank"]), "%", &source),
        evidence("E6", "Net GEX", chain.quality.gex_ready.then_some(chain.metrics.net_gex).flatten(), "USD gamma", &source),
        evidence("E7", "Gamma Flip", chain.quality.gex_ready.then_some(chain.metrics.gamma_flip).flatten(), "USD", &source),
        evidence("E8", "Quote Coverage", Some(chain.quality.quote_coverage_pct), "%", &source),
        evidence("E9", "Fresh Quote Coverage", Some(chain.quality.fresh_quote_coverage_pct), "%", &source),
        evidence("E10", "Usable Rows", Some(chain.quality.usable_pct), "%", &source),
    ]
}

fn evidence(id: &str, label: &str, value: Option<f64>, unit: &str, source: &str) -> EvidenceItem {
    EvidenceItem {
        id: id.into(),
        label: label.into(),
        value,
        unit: unit.into(),
        status: if value.is_some() { "available" } else { "unavailable" },
        source: source.into(),
    }
}

fn value_at(value: &Value, path: &[&str]) -> Option<f64> {
    let mut current = value;
    for segment in path { current = current.get(*segment)?; }
    current.as_f64()
}

fn render_answer(intent: &str, chain: &ChainSnapshot, volatility: &Value) -> String {
    match intent {
        "volatility" => {
            let atm = format_metric(chain.metrics.atm_iv, "%");
            let rv20 = format_metric(value_at(volatility, &["realized_volatility", "20"]), "%");
            let vrp = value_at(volatility, &["vrp20"]);
            let rank = format_metric(value_at(volatility, &["iv_rank"]), "%");
            let conclusion = match vrp {
                Some(value) if value > 2.0 => "当前隐含波动率高于近 20 日实现波动率，但这只是相对定价信号，不等同于做空波动率建议。",
                Some(value) if value < -2.0 => "当前隐含波动率低于近 20 日实现波动率，但仍需结合事件风险和期限结构。",
                Some(_) => "当前 IV 与 RV20 的差距较小，没有明显的相对波动率溢价。",
                None => "缺少可用的 RV20/VRP20，无法判断 IV 是否相对偏贵。",
            };
            format!("{} 当前 ATM IV 为 {atm} [E2]，RV20 为 {rv20} [E3]，VRP20 为 {} [E4]，IV Rank 为 {rank} [E5]。{conclusion}", chain.symbol, format_metric(vrp, " vol pts"))
        }
        "dealer_exposure" => {
            if !chain.quality.gex_ready {
                format!("当前 GEX 质量门未通过，因此不应形成 dealer-gamma 判断。Net GEX 与 Gamma Flip 均视为不可用 [E6][E7]；新鲜报价覆盖率为 {} [E9]。", format_metric(Some(chain.quality.fresh_quote_coverage_pct), "%"))
            } else {
                let regime = match chain.metrics.net_gex {
                    Some(value) if value > 0.0 => "正 gamma",
                    Some(value) if value < 0.0 => "负 gamma",
                    _ => "中性或未知 gamma",
                };
                format!("按当前 {} dealer model，截面处于{regime}状态：Net GEX 为 {} [E6]，Gamma Flip 为 {} [E7]，现货为 {} [E1]。这是模型化敞口，不是可观测的真实 dealer 持仓。", chain.dealer_model, format_metric(chain.metrics.net_gex, ""), format_metric(chain.metrics.gamma_flip, ""), format_metric(Some(chain.spot), ""))
            }
        }
        "data_quality" => format!(
            "这个截面的报价覆盖率为 {} [E8]，新鲜报价覆盖率为 {} [E9]，可用行比例为 {} [E10]。{}",
            format_metric(Some(chain.quality.quote_coverage_pct), "%"),
            format_metric(Some(chain.quality.fresh_quote_coverage_pct), "%"),
            format_metric(Some(chain.quality.usable_pct), "%"),
            if chain.quality.blocked_metrics.is_empty() { "当前没有被质量门阻止的指标。".into() } else { format!("被阻止的指标：{}。", chain.quality.blocked_metrics.join("、")) }
        ),
        _ => format!("{} 在 {} ET 的现货为 {} [E1]，ATM IV 为 {} [E2]，Net GEX 为 {} [E6]。当前报价覆盖率 {} [E8]。你可以继续问“IV 贵吗”“Gamma 状态如何”或“这些数据可靠吗”。", chain.symbol, chain.minute, format_metric(Some(chain.spot), ""), format_metric(chain.metrics.atm_iv, "%"), format_metric(chain.metrics.net_gex, ""), format_metric(Some(chain.quality.quote_coverage_pct), "%")),
    }
}

fn format_metric(value: Option<f64>, suffix: &str) -> String {
    value.map(|number| format!("{number:.2}{suffix}")).unwrap_or_else(|| "不可用".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_supported_research_questions() {
        assert_eq!(classify_intent("现在 IV 贵吗？"), "volatility");
        assert_eq!(classify_intent("What is the gamma flip?"), "dealer_exposure");
        assert_eq!(classify_intent("这些数据可靠吗"), "data_quality");
        assert_eq!(classify_intent("总结一下"), "overview");
    }

    #[test]
    fn rejects_empty_and_oversized_questions() {
        let mut request = CopilotRequest { question: " ".into(), mode: "replay".into(), symbol: "SPY".into(), date: None, minute: None, expiration: None, pricing_mode: default_pricing_mode(), dealer_model: default_dealer_model() };
        assert!(request.validate().is_err());
        request.question = "x".repeat(MAX_QUESTION_CHARS + 1);
        assert!(request.validate().is_err());
    }

    #[test]
    fn nested_value_lookup_is_missing_safe() {
        let value = serde_json::json!({"realized_volatility": {"20": 18.25}});
        assert_eq!(value_at(&value, &["realized_volatility", "20"]), Some(18.25));
        assert_eq!(value_at(&value, &["vrp20"]), None);
    }
}
