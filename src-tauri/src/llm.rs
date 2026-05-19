use crate::models::{DateSource, PreviewEntry};
use chrono::{NaiveDate, NaiveTime};
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct LlmEnvelope {
    entries: Option<Vec<LlmEntry>>,
}

#[derive(Debug, Deserialize)]
struct LlmEntry {
    date_expression: Option<String>,
    due_date: String,
    due_time: Option<String>,
    item: String,
    warning: Option<String>,
}

pub fn parse_llm_entries(source_text: &str, json: &str) -> Result<Vec<PreviewEntry>, String> {
    let envelope: LlmEnvelope =
        serde_json::from_str(json).map_err(|err| format!("LLM response JSON is invalid: {err}"))?;
    let entries = envelope
        .entries
        .ok_or_else(|| "LLM response is missing entries".to_string())?;

    if entries.is_empty() {
        return Err("LLM response entries is empty".to_string());
    }

    entries
        .into_iter()
        .enumerate()
        .map(|(index, entry)| {
            let title = entry.item.trim().to_string();
            if title.is_empty() {
                return Err(format!("LLM entry {index} item is empty"));
            }

            let due_date = NaiveDate::parse_from_str(&entry.due_date, "%Y-%m-%d")
                .map_err(|err| format!("LLM entry {index} due_date is invalid: {err}"))?;
            let due_time = parse_due_time(index, entry.due_time)?;

            Ok(PreviewEntry {
                title,
                due_date,
                due_time,
                source_text: source_text.to_string(),
                date_expression: entry.date_expression,
                date_source: DateSource::Llm,
                warning: entry.warning,
            })
        })
        .collect()
}

fn parse_due_time(index: usize, value: Option<String>) -> Result<Option<NaiveTime>, String> {
    match value {
        Some(value) if !value.trim().is_empty() => NaiveTime::parse_from_str(value.trim(), "%H:%M")
            .map(Some)
            .map_err(|err| format!("LLM entry {index} due_time is invalid: {err}")),
        _ => Ok(None),
    }
}

#[derive(Debug, Clone)]
pub struct ProviderRequest {
    pub provider: String,
    pub api_key: String,
    pub model: String,
    pub prompt: String,
    pub user_text: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct ChatMessage {
    content: Option<String>,
}

pub async fn call_provider(request: ProviderRequest) -> Result<String, String> {
    let api_key = request.api_key.trim();
    if api_key.is_empty() {
        return Err("LLM API key is blank".to_string());
    }

    let endpoint = endpoint_for_provider(&request.provider)?;
    let body = provider_body(&request)?;

    let client = reqwest::Client::new();
    let response = client
        .post(endpoint)
        .bearer_auth(api_key)
        .header("Content-Type", "application/json; charset=utf-8")
        .header("HTTP-Referer", "http://localhost/todo-float")
        .header("X-Title", "Todo Float")
        .json(&body)
        .send()
        .await
        .map_err(|err| sanitize_error(api_key, format!("LLM request failed: {err}")))?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|err| sanitize_error(api_key, format!("LLM response body read failed: {err}")))?;

    if !status.is_success() {
        return Err(provider_http_error(status, &text));
    }

    parse_chat_content(&text)
}

fn endpoint_for_provider(provider: &str) -> Result<&'static str, String> {
    match provider {
        "openrouter" => Ok("https://openrouter.ai/api/v1/chat/completions"),
        "bigmodel" => Ok("https://open.bigmodel.cn/api/paas/v4/chat/completions"),
        other => Err(format!("Unsupported LLM provider: {other}")),
    }
}

fn provider_body(request: &ProviderRequest) -> Result<Value, String> {
    endpoint_for_provider(&request.provider)?;

    let mut body = serde_json::json!({
        "model": request.model,
        "messages": [
            { "role": "system", "content": request.prompt },
            { "role": "user", "content": request.user_text }
        ],
        "stream": false,
        "response_format": { "type": "json_object" },
        "temperature": 0,
        "max_tokens": 2048
    });

    match request.provider.as_str() {
        "openrouter" => {
            body["reasoning"] = serde_json::json!({ "effort": "none", "exclude": true });
        }
        "bigmodel" => {
            body["thinking"] = serde_json::json!({ "type": "disabled" });
        }
        _ => unreachable!("provider already validated"),
    }

    Ok(body)
}

fn provider_http_error(status: StatusCode, _body: &str) -> String {
    format!("LLM returned HTTP {status}")
}

fn parse_chat_content(text: &str) -> Result<String, String> {
    let parsed: ChatResponse = serde_json::from_str(text)
        .map_err(|err| format!("LLM response structure is invalid: {err}"))?;

    parsed
        .choices
        .into_iter()
        .next()
        .and_then(|choice| choice.message.content)
        .ok_or_else(|| "LLM response is missing choices[0].message.content".to_string())
}

fn sanitize_error(api_key: &str, message: String) -> String {
    if api_key.is_empty() {
        message
    } else {
        message.replace(api_key, "[redacted]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_entries() {
        let json = r#"{
          "entries": [
            {
              "date_expression": "tomorrow",
              "due_date": "2026-05-20",
              "due_time": "09:30",
              "item": "  Pick up files  ",
              "date_confidence": "high",
              "warning": "Check office hours"
            }
          ],
          "notes": null
        }"#;

        let entries = parse_llm_entries("tomorrow pick up files", json).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].title, "Pick up files");
        assert_eq!(
            entries[0].due_date,
            NaiveDate::from_ymd_opt(2026, 5, 20).unwrap()
        );
        assert_eq!(
            entries[0].due_time,
            Some(NaiveTime::from_hms_opt(9, 30, 0).unwrap())
        );
        assert_eq!(entries[0].source_text, "tomorrow pick up files");
        assert_eq!(entries[0].date_expression.as_deref(), Some("tomorrow"));
        assert_eq!(entries[0].date_source, DateSource::Llm);
        assert_eq!(entries[0].warning.as_deref(), Some("Check office hours"));
    }

    #[test]
    fn rejects_invalid_due_date() {
        let json = r#"{
          "entries": [
            {
              "date_expression": "tomorrow",
              "due_date": "tomorrow",
              "due_time": null,
              "item": "Pick up files",
              "warning": null
            }
          ]
        }"#;

        let err = parse_llm_entries("tomorrow pick up files", json).unwrap_err();

        assert!(err.contains("due_date"));
    }

    #[test]
    fn rejects_empty_item() {
        let json = r#"{
          "entries": [
            {
              "date_expression": "tomorrow",
              "due_date": "2026-05-20",
              "due_time": null,
              "item": " ",
              "warning": null
            }
          ]
        }"#;

        let err = parse_llm_entries("tomorrow", json).unwrap_err();

        assert!(err.contains("item"));
    }

    #[test]
    fn rejects_invalid_due_time() {
        let json = r#"{
          "entries": [
            {
              "date_expression": "tomorrow",
              "due_date": "2026-05-20",
              "due_time": "25:61",
              "item": "Pick up files",
              "warning": null
            }
          ]
        }"#;

        let err = parse_llm_entries("tomorrow pick up files", json).unwrap_err();

        assert!(err.contains("due_time"));
    }

    #[test]
    fn treats_empty_due_time_as_none() {
        let json = r#"{
          "entries": [
            {
              "date_expression": "tomorrow",
              "due_date": "2026-05-20",
              "due_time": "",
              "item": "Pick up files",
              "warning": null
            }
          ]
        }"#;

        let entries = parse_llm_entries("tomorrow pick up files", json).unwrap();

        assert_eq!(entries[0].due_time, None);
    }

    #[test]
    fn rejects_missing_entries() {
        let err = parse_llm_entries("tomorrow", r#"{"notes":null}"#).unwrap_err();

        assert!(err.contains("entries"));
    }

    #[test]
    fn rejects_empty_entries() {
        let err = parse_llm_entries("tomorrow", r#"{"entries":[]}"#).unwrap_err();

        assert!(err.contains("entries"));
    }

    #[test]
    fn http_error_does_not_return_provider_body() {
        let body = r#"{"error":"sk-live-secret-looking-value"}"#;

        let err = provider_http_error(StatusCode::UNAUTHORIZED, body);

        assert_eq!(err, "LLM returned HTTP 401 Unauthorized");
        assert!(!err.contains("sk-live-secret-looking-value"));
        assert!(!err.contains("error"));
    }

    #[test]
    fn malformed_chat_response_does_not_return_raw_body() {
        let body = r#"{"error":"sk-live-secret-looking-value"}"#;

        let err = parse_chat_content(body).unwrap_err();

        assert!(err.contains("LLM response structure is invalid"));
        assert!(!err.contains("sk-live-secret-looking-value"));
        assert!(!err.contains(body));
    }

    #[test]
    fn openrouter_body_uses_reasoning_without_thinking() {
        let body = provider_body(&provider_request("openrouter")).unwrap();

        assert_eq!(body["model"], "test-model");
        assert_eq!(body["messages"][0]["role"], "system");
        assert_eq!(body["messages"][0]["content"], "system prompt");
        assert_eq!(body["messages"][1]["role"], "user");
        assert_eq!(body["messages"][1]["content"], "user text");
        assert_eq!(body["stream"], false);
        assert_eq!(body["response_format"]["type"], "json_object");
        assert_eq!(body["temperature"], 0);
        assert_eq!(body["max_tokens"], 2048);
        assert_eq!(body["reasoning"]["effort"], "none");
        assert_eq!(body["reasoning"]["exclude"], true);
        assert!(body.get("thinking").is_none());
    }

    #[test]
    fn bigmodel_body_uses_thinking_without_reasoning() {
        let body = provider_body(&provider_request("bigmodel")).unwrap();

        assert_eq!(body["model"], "test-model");
        assert_eq!(body["messages"][0]["role"], "system");
        assert_eq!(body["messages"][0]["content"], "system prompt");
        assert_eq!(body["messages"][1]["role"], "user");
        assert_eq!(body["messages"][1]["content"], "user text");
        assert_eq!(body["stream"], false);
        assert_eq!(body["response_format"]["type"], "json_object");
        assert_eq!(body["temperature"], 0);
        assert_eq!(body["max_tokens"], 2048);
        assert_eq!(body["thinking"]["type"], "disabled");
        assert!(body.get("reasoning").is_none());
    }

    fn provider_request(provider: &str) -> ProviderRequest {
        ProviderRequest {
            provider: provider.to_string(),
            api_key: "test-key".to_string(),
            model: "test-model".to_string(),
            prompt: "system prompt".to_string(),
            user_text: "user text".to_string(),
        }
    }
}
