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
