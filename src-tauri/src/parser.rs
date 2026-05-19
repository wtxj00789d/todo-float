use crate::date_rules::resolve_date;
use crate::models::{DateSource, PreviewEntry};
use chrono::{Datelike, Duration, NaiveDate, Weekday};

pub fn build_prompt(base: NaiveDate) -> String {
    let current_week_start = week_start(base);
    let next_week_start = current_week_start + Duration::days(7);
    let current_week_table = week_table(current_week_start);
    let next_week_table = week_table(next_week_start);

    format!(
        r#"你是 Todo Float 的中文待办解析器。请把用户输入拆分成待办事项，并抽取日期、时间和事项内容。

只输出一个 JSON object，不要输出 markdown、代码块、解释、前后缀或多余文本。

当前基准日期：{base}
时区：Asia/Shanghai
用户习惯：一周从周一开始，周日结束；周一是每周第一天，周日是每周最后一天。

本周日期表（周一开始）：
{current_week_table}

下周日期表（周一开始）：
{next_week_table}

确定性日期规则：
- 今天/明天/后天/大后天：分别解析为基准日期 +0/+1/+2/+3 天。
- 裸星期表达式（如周一、星期二、礼拜三）：按当前周解析；即使日期已经过去，也保持当前周日期，并在 warning 提醒用户确认。
- 下周/下星期/下礼拜：按下一个周一开始的周解析；下下周/下下星期/下下礼拜：按再下一周解析。
- “周五之前”“星期五前”等截止表达式按周五当天解析，不提前到周四。
- “下周末”默认解析为下周六；只有明确写出周日/星期日/礼拜天时才解析为下周日。
- 月底解析为当月最后一天；月初解析为当月 1 号，若已过去则 warning 提醒。
- 下个月3号、下个月三号等解析为下个月对应日期。
- 5月20号、5月20日、十一月三号等月日表达式默认使用基准日期所在年份；若已过去则 warning 提醒。

拆分与清洗规则：
- 用户一句话里可能有多个待办，用逗号、顿号、分号、换行、以及“还有/另外/顺便/并且”等连接词拆分。
- 去掉触发词和口语前缀，例如“提醒我”“帮我记一下”“记得”“到时候”“需要”“我要”。
- item 只保留待办本身，不要把日期表达式、时间表达式或触发词留在事项里。
- date_expression 保留用户原文中的日期短语；没有日期短语时为 null。
- due_time 只在用户明确给出时间时填写 HH:mm，否则为 null。
- 不确定、日期已过去、语义含糊时，把简短中文提示写入 warning；没有 warning 时为 null。

输出字段必须严格为：
{{
  "entries": [
    {{
      "date_expression": string|null,
      "due_date": "YYYY-MM-DD",
      "due_time": "HH:mm"|null,
      "item": string,
      "warning": string|null
    }}
  ],
  "notes": string|null
}}"#,
        base = base,
        current_week_table = current_week_table,
        next_week_table = next_week_table
    )
}

pub fn apply_local_overrides(
    mut entries: Vec<PreviewEntry>,
    source_text: &str,
    base: NaiveDate,
) -> Vec<PreviewEntry> {
    for entry in &mut entries {
        let resolution = match entry.date_expression.as_deref().map(str::trim) {
            Some(expression) if !expression.is_empty() => {
                resolve_date(expression, base)
                    .or_else(|| resolve_unambiguous_source(source_text, base))
            }
            _ => resolve_unambiguous_source(source_text, base),
        };

        if let Some(resolution) = resolution {
            entry.due_date = resolution.date;
            entry.date_expression = Some(resolution.expression);
            entry.date_source = DateSource::Rule;
            if resolution.warning.is_some() {
                entry.warning = resolution.warning;
            }
        }
    }

    entries
}

fn resolve_unambiguous_source(
    source_text: &str,
    base: NaiveDate,
) -> Option<crate::date_rules::DateResolution> {
    if local_date_expression_count(source_text, base) == 1 {
        resolve_date(source_text, base)
    } else {
        None
    }
}

fn local_date_expression_count(source_text: &str, base: NaiveDate) -> usize {
    split_date_clauses(source_text)
        .into_iter()
        .filter(|clause| resolve_date(clause, base).is_some())
        .count()
}

fn split_date_clauses(source_text: &str) -> Vec<&str> {
    let mut clauses = vec![source_text];
    for connector in ["还有", "另外", "顺便", "并且"] {
        clauses = clauses
            .into_iter()
            .flat_map(|clause| clause.split(connector))
            .collect();
    }

    clauses
        .into_iter()
        .flat_map(|clause| {
            clause.split(|ch| matches!(ch, '，' | '、' | '；' | ';' | ',' | '\n' | '\r'))
        })
        .map(str::trim)
        .filter(|clause| !clause.is_empty())
        .collect()
}

fn week_start(base: NaiveDate) -> NaiveDate {
    let offset = match base.weekday() {
        Weekday::Mon => 0,
        Weekday::Tue => 1,
        Weekday::Wed => 2,
        Weekday::Thu => 3,
        Weekday::Fri => 4,
        Weekday::Sat => 5,
        Weekday::Sun => 6,
    };
    base - Duration::days(offset)
}

fn week_table(start: NaiveDate) -> String {
    ["一", "二", "三", "四", "五", "六", "日"]
        .into_iter()
        .enumerate()
        .map(|(index, day)| {
            let date = start + Duration::days(index as i64);
            format!("周{day}={date}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime;

    fn base() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 5, 19).unwrap()
    }

    fn preview_entry(date_expression: Option<&str>, warning: Option<&str>) -> PreviewEntry {
        PreviewEntry {
            title: "提交材料".to_string(),
            due_date: NaiveDate::from_ymd_opt(2026, 5, 20).unwrap(),
            due_time: Some(NaiveTime::from_hms_opt(9, 30, 0).unwrap()),
            source_text: "提醒我下周二提交材料".to_string(),
            date_expression: date_expression.map(str::to_string),
            date_source: DateSource::Llm,
            warning: warning.map(str::to_string),
        }
    }

    #[test]
    fn build_prompt_includes_base_week_tables_and_week_start_rule() {
        let prompt = build_prompt(base());

        assert!(prompt.contains("当前基准日期：2026-05-19"));
        assert!(prompt.contains("本周日期表（周一开始）："));
        assert!(prompt.contains("周二=2026-05-19"));
        assert!(prompt.contains("下周日期表（周一开始）："));
        assert!(prompt.contains("周二=2026-05-26"));
        assert!(prompt.contains("一周从周一开始，周日结束"));
    }

    #[test]
    fn apply_local_overrides_uses_rule_date_when_expression_matches() {
        let entries = vec![preview_entry(Some("下周二"), Some("LLM warning"))];

        let entries = apply_local_overrides(entries, "提醒我下周二提交材料", base());

        assert_eq!(
            entries[0].due_date,
            NaiveDate::from_ymd_opt(2026, 5, 26).unwrap()
        );
        assert_eq!(entries[0].date_expression.as_deref(), Some("下周二"));
        assert_eq!(entries[0].date_source, DateSource::Rule);
        assert_eq!(entries[0].warning.as_deref(), Some("LLM warning"));
    }

    #[test]
    fn apply_local_overrides_prefers_entry_expression_over_source_text() {
        let entries = vec![preview_entry(Some("下周二"), None)];

        let entries = apply_local_overrides(entries, "明天买菜，提醒我下周二提交材料", base());

        assert_eq!(
            entries[0].due_date,
            NaiveDate::from_ymd_opt(2026, 5, 26).unwrap()
        );
        assert_eq!(entries[0].date_expression.as_deref(), Some("下周二"));
        assert_eq!(entries[0].date_source, DateSource::Rule);
    }

    #[test]
    fn apply_local_overrides_does_not_use_multi_date_source_without_expression() {
        let mut entry = preview_entry(None, Some("LLM warning"));
        entry.due_date = NaiveDate::from_ymd_opt(2026, 5, 27).unwrap();
        let entries = vec![entry];

        let entries = apply_local_overrides(entries, "明天买菜，下周二提交材料", base());

        assert_eq!(
            entries[0].due_date,
            NaiveDate::from_ymd_opt(2026, 5, 27).unwrap()
        );
        assert_eq!(entries[0].date_expression, None);
        assert_eq!(entries[0].date_source, DateSource::Llm);
        assert_eq!(entries[0].warning.as_deref(), Some("LLM warning"));
    }

    #[test]
    fn apply_local_overrides_does_not_use_connector_multi_date_source_without_expression() {
        let mut entry = preview_entry(None, Some("LLM warning"));
        entry.due_date = NaiveDate::from_ymd_opt(2026, 5, 27).unwrap();
        let entries = vec![entry];

        let entries = apply_local_overrides(entries, "明天买菜还有下周二提交材料", base());

        assert_eq!(
            entries[0].due_date,
            NaiveDate::from_ymd_opt(2026, 5, 27).unwrap()
        );
        assert_eq!(entries[0].date_expression, None);
        assert_eq!(entries[0].date_source, DateSource::Llm);
        assert_eq!(entries[0].warning.as_deref(), Some("LLM warning"));
    }

    #[test]
    fn apply_local_overrides_uses_single_date_source_without_expression() {
        let mut entry = preview_entry(None, None);
        entry.due_date = NaiveDate::from_ymd_opt(2026, 5, 27).unwrap();
        let entries = vec![entry];

        let entries = apply_local_overrides(entries, "明天提醒我拿文件", base());

        assert_eq!(
            entries[0].due_date,
            NaiveDate::from_ymd_opt(2026, 5, 20).unwrap()
        );
        assert_eq!(entries[0].date_expression.as_deref(), Some("明天"));
        assert_eq!(entries[0].date_source, DateSource::Rule);
    }

    #[test]
    fn apply_local_overrides_preserves_local_rule_warning() {
        let entries = vec![preview_entry(Some("周一"), Some("LLM warning"))];

        let entries = apply_local_overrides(entries, "提醒我周一补发票", base());

        assert_eq!(
            entries[0].due_date,
            NaiveDate::from_ymd_opt(2026, 5, 18).unwrap()
        );
        assert_eq!(entries[0].date_expression.as_deref(), Some("周一"));
        assert_eq!(entries[0].date_source, DateSource::Rule);
        assert!(entries[0].warning.as_deref().unwrap().contains("已过去"));
    }
}
