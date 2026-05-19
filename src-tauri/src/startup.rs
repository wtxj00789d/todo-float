use crate::models::Todo;

pub fn should_show_popup(
    todos: &[Todo],
    latest_snapshot: Option<(Option<String>, bool)>,
) -> bool {
    let Some(current_max_created_at) = todos.iter().map(|todo| todo.created_at.as_str()).max()
    else {
        return false;
    };

    let Some((snapshot_max_created_at, _suppressed)) = latest_snapshot else {
        return true;
    };

    let Some(snapshot_max_created_at) = snapshot_max_created_at else {
        return true;
    };

    current_max_created_at > snapshot_max_created_at.as_str()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DateSource;
    use chrono::NaiveDate;

    #[test]
    fn empty_day_false() {
        assert!(!should_show_popup(&[], None));
    }

    #[test]
    fn no_prior_snapshot_true() {
        let todos = vec![todo("2026-05-19T09:00:00Z")];

        assert!(should_show_popup(&todos, None));
    }

    #[test]
    fn snapshot_current_false() {
        let todos = vec![
            todo("2026-05-19T09:00:00Z"),
            todo("2026-05-19T10:00:00Z"),
        ];

        assert!(!should_show_popup(
            &todos,
            Some((Some("2026-05-19T10:00:00Z".to_string()), false))
        ));
    }

    #[test]
    fn new_todo_after_snapshot_true() {
        let todos = vec![
            todo("2026-05-19T09:00:00Z"),
            todo("2026-05-19T11:00:00Z"),
        ];

        assert!(should_show_popup(
            &todos,
            Some((Some("2026-05-19T10:00:00Z".to_string()), false))
        ));
    }

    #[test]
    fn suppressed_snapshot_with_no_new_todo_false() {
        let todos = vec![todo("2026-05-19T09:00:00Z")];

        assert!(!should_show_popup(
            &todos,
            Some((Some("2026-05-19T09:00:00Z".to_string()), true))
        ));
    }

    #[test]
    fn suppressed_snapshot_with_new_todo_true() {
        let todos = vec![
            todo("2026-05-19T09:00:00Z"),
            todo("2026-05-19T10:00:00Z"),
        ];

        assert!(should_show_popup(
            &todos,
            Some((Some("2026-05-19T09:00:00Z".to_string()), true))
        ));
    }

    #[test]
    fn snapshot_without_max_created_at_true() {
        let todos = vec![todo("2026-05-19T09:00:00Z")];

        assert!(should_show_popup(&todos, Some((None, false))));
    }

    fn todo(created_at: &str) -> Todo {
        Todo {
            id: created_at.to_string(),
            title: "Test todo".to_string(),
            due_date: NaiveDate::from_ymd_opt(2026, 5, 19).unwrap(),
            due_time: None,
            source_text: None,
            date_expression: None,
            date_source: DateSource::Rule,
            warning: None,
            created_at: created_at.to_string(),
            updated_at: created_at.to_string(),
            deleted_at: None,
            completed_at: None,
        }
    }
}
