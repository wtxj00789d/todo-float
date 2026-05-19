use crate::models::{DateSource, Todo};
use chrono::{NaiveDate, NaiveTime};
use rusqlite::types::Type;
use rusqlite::{params, Connection};
use std::error::Error;
use uuid::Uuid;

const DATE_FORMAT: &str = "%Y-%m-%d";
const TIME_FORMAT: &str = "%H:%M";

pub fn open_database(path: &std::path::Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    migrate(&conn)?;
    Ok(conn)
}

pub fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS todos (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            due_date TEXT NOT NULL,
            due_time TEXT NULL,
            source_text TEXT NULL,
            date_expression TEXT NULL,
            date_source TEXT NOT NULL,
            warning TEXT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT NULL,
            completed_at TEXT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_todos_due_date_active
        ON todos(due_date, deleted_at, completed_at);

        CREATE TABLE IF NOT EXISTS popup_events (
            id TEXT PRIMARY KEY,
            date TEXT NOT NULL,
            shown_at TEXT NOT NULL,
            todo_snapshot_max_created_at TEXT NULL,
            suppressed INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_popup_events_date_shown
        ON popup_events(date, shown_at);
        "#,
    )
}

pub fn insert_todo(
    conn: &Connection,
    title: &str,
    due_date: NaiveDate,
) -> rusqlite::Result<String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        r#"
        INSERT INTO todos (
            id, title, due_date, due_time, source_text, date_expression, date_source,
            warning, created_at, updated_at, deleted_at, completed_at
        ) VALUES (?1, ?2, ?3, NULL, NULL, NULL, 'rule', NULL, ?4, ?4, NULL, NULL)
        "#,
        params![id, title, due_date.format(DATE_FORMAT).to_string(), now],
    )?;
    Ok(id)
}

pub fn active_todos_for_date(
    conn: &Connection,
    due_date: NaiveDate,
) -> rusqlite::Result<Vec<Todo>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, title, due_date, due_time, source_text, date_expression, date_source,
               warning, created_at, updated_at, deleted_at, completed_at
        FROM todos
        WHERE due_date = ?1 AND deleted_at IS NULL AND completed_at IS NULL
        ORDER BY created_at ASC
        "#,
    )?;
    let rows = stmt.query_map(params![due_date.format(DATE_FORMAT).to_string()], |row| {
        Ok(Todo {
            id: row.get(0)?,
            title: row.get(1)?,
            due_date: parse_date_column(row.get(2)?, 2)?,
            due_time: parse_optional_time_column(row.get(3)?, 3)?,
            source_text: row.get(4)?,
            date_expression: row.get(5)?,
            date_source: parse_date_source(row.get(6)?, 6)?,
            warning: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
            deleted_at: row.get(10)?,
            completed_at: row.get(11)?,
        })
    })?;

    rows.collect()
}

pub fn record_popup(
    conn: &Connection,
    date: NaiveDate,
    max_created_at: Option<&str>,
    suppressed: bool,
) -> rusqlite::Result<()> {
    let id = Uuid::new_v4().to_string();
    let shown_at = chrono::Utc::now().to_rfc3339();
    conn.execute(
        r#"
        INSERT INTO popup_events (id, date, shown_at, todo_snapshot_max_created_at, suppressed)
        VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
        params![
            id,
            date.format(DATE_FORMAT).to_string(),
            shown_at,
            max_created_at,
            if suppressed { 1 } else { 0 }
        ],
    )?;
    Ok(())
}

pub fn latest_popup_snapshot(
    conn: &Connection,
    date: NaiveDate,
) -> rusqlite::Result<Option<(Option<String>, bool)>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT todo_snapshot_max_created_at, suppressed
        FROM popup_events
        WHERE date = ?1
        ORDER BY shown_at DESC
        LIMIT 1
        "#,
    )?;
    let mut rows = stmt.query(params![date.format(DATE_FORMAT).to_string()])?;
    if let Some(row) = rows.next()? {
        let snapshot: Option<String> = row.get(0)?;
        let suppressed: i64 = row.get(1)?;
        Ok(Some((snapshot, suppressed == 1)))
    } else {
        Ok(None)
    }
}

fn parse_date_column(value: String, column: usize) -> rusqlite::Result<NaiveDate> {
    NaiveDate::parse_from_str(&value, DATE_FORMAT)
        .map_err(|err| rusqlite::Error::FromSqlConversionFailure(column, Type::Text, Box::new(err)))
}

fn parse_optional_time_column(
    value: Option<String>,
    column: usize,
) -> rusqlite::Result<Option<NaiveTime>> {
    value
        .map(|value| {
            NaiveTime::parse_from_str(&value, TIME_FORMAT).map_err(|err| {
                rusqlite::Error::FromSqlConversionFailure(column, Type::Text, Box::new(err))
            })
        })
        .transpose()
}

fn parse_date_source(value: String, column: usize) -> rusqlite::Result<DateSource> {
    match value.as_str() {
        "rule" => Ok(DateSource::Rule),
        "llm" => Ok(DateSource::Llm),
        "default" => Ok(DateSource::Default),
        _ => Err(rusqlite::Error::FromSqlConversionFailure(
            column,
            Type::Text,
            Box::<dyn Error + Send + Sync>::from(format!("invalid date_source: {value}")),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrates_and_returns_today_todos() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let today = NaiveDate::from_ymd_opt(2026, 5, 19).unwrap();
        let tomorrow = NaiveDate::from_ymd_opt(2026, 5, 20).unwrap();

        insert_todo(&conn, "拿文件", today).unwrap();
        insert_todo(&conn, "找凯莉", today).unwrap();
        insert_todo(&conn, "买牛奶", tomorrow).unwrap();

        let todos = active_todos_for_date(&conn, today).unwrap();
        assert_eq!(todos.len(), 2);
        assert_eq!(todos[0].title, "拿文件");
        assert_eq!(todos[1].title, "找凯莉");
    }

    #[test]
    fn stores_popup_snapshot() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let today = NaiveDate::from_ymd_opt(2026, 5, 19).unwrap();

        record_popup(&conn, today, Some("2026-05-19T09:00:00Z"), false).unwrap();
        let snapshot = latest_popup_snapshot(&conn, today).unwrap().unwrap();

        assert_eq!(snapshot.0.unwrap(), "2026-05-19T09:00:00Z");
        assert!(!snapshot.1);
    }

    #[test]
    fn maps_due_time_as_hour_minute_text() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let today = NaiveDate::from_ymd_opt(2026, 5, 19).unwrap();
        let now = "2026-05-19T09:00:00Z";

        conn.execute(
            r#"
            INSERT INTO todos (
                id, title, due_date, due_time, source_text, date_expression, date_source,
                warning, created_at, updated_at, deleted_at, completed_at
            ) VALUES (?1, '带时间事项', ?2, '09:30', NULL, NULL, 'rule', NULL, ?3, ?3, NULL, NULL)
            "#,
            params![Uuid::new_v4().to_string(), today.to_string(), now],
        )
        .unwrap();

        let stored: String = conn
            .query_row("SELECT due_time FROM todos LIMIT 1", [], |row| row.get(0))
            .unwrap();
        let todos = active_todos_for_date(&conn, today).unwrap();

        assert_eq!(stored, "09:30");
        assert_eq!(
            todos[0].due_time,
            Some(NaiveTime::from_hms_opt(9, 30, 0).unwrap())
        );
    }
}
