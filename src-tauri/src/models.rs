use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DateSource {
    Rule,
    Llm,
    Default,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub due_date: NaiveDate,
    #[serde(with = "optional_hour_minute_time")]
    pub due_time: Option<NaiveTime>,
    pub source_text: Option<String>,
    pub date_expression: Option<String>,
    pub date_source: DateSource,
    pub warning: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PreviewEntry {
    pub title: String,
    pub due_date: NaiveDate,
    #[serde(with = "optional_hour_minute_time")]
    pub due_time: Option<NaiveTime>,
    pub source_text: String,
    pub date_expression: Option<String>,
    pub date_source: DateSource,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppState {
    pub today: NaiveDate,
    pub todos: Vec<Todo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParseRequest {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParseResponse {
    pub entries: Vec<PreviewEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SavePreviewRequest {
    pub entries: Vec<PreviewEntry>,
}

mod optional_hour_minute_time {
    use chrono::NaiveTime;
    use serde::{Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%H:%M";

    pub fn serialize<S>(time: &Option<NaiveTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match time {
            Some(time) => serializer.serialize_some(&time.format(FORMAT).to_string()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Option::<String>::deserialize(deserializer)?;
        value
            .map(|value| NaiveTime::parse_from_str(&value, FORMAT).map_err(serde::de::Error::custom))
            .transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn preview_entry() -> PreviewEntry {
        PreviewEntry {
            title: "拿文件".to_string(),
            due_date: NaiveDate::from_ymd_opt(2026, 5, 20).unwrap(),
            due_time: Some(NaiveTime::from_hms_opt(9, 30, 0).unwrap()),
            source_text: "明天九点半拿文件".to_string(),
            date_expression: Some("明天".to_string()),
            date_source: DateSource::Rule,
            warning: None,
        }
    }

    #[test]
    fn serializes_preview_entry_due_time_as_hour_minute() {
        let value = serde_json::to_value(preview_entry()).unwrap();

        assert_eq!(value["due_time"], json!("09:30"));
    }

    #[test]
    fn deserializes_preview_entry_due_time_from_hour_minute() {
        let entry: PreviewEntry = serde_json::from_value(json!({
            "title": "拿文件",
            "due_date": "2026-05-20",
            "due_time": "09:30",
            "source_text": "明天九点半拿文件",
            "date_expression": "明天",
            "date_source": "rule",
            "warning": null
        }))
        .unwrap();

        assert_eq!(entry.due_time, Some(NaiveTime::from_hms_opt(9, 30, 0).unwrap()));
    }

    #[test]
    fn deserializes_preview_entry_null_due_time() {
        let entry: PreviewEntry = serde_json::from_value(json!({
            "title": "拿文件",
            "due_date": "2026-05-20",
            "due_time": null,
            "source_text": "明天拿文件",
            "date_expression": "明天",
            "date_source": "rule",
            "warning": null
        }))
        .unwrap();

        assert_eq!(entry.due_time, None);
    }
}
