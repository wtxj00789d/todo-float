use chrono::{Datelike, Duration, NaiveDate, Weekday};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateResolution {
    pub date: NaiveDate,
    pub expression: String,
    pub warning: Option<String>,
}

pub fn resolve_date(input: &str, base: NaiveDate) -> Option<DateResolution> {
    let text = input.trim();

    for (expr, days) in [("大后天", 3), ("后天", 2), ("明天", 1), ("今天", 0)] {
        if text.contains(expr) {
            return Some(DateResolution {
                date: base + Duration::days(days),
                expression: expr.to_string(),
                warning: None,
            });
        }
    }

    if text.contains("月底") {
        let first_next_month = if base.month() == 12 {
            NaiveDate::from_ymd_opt(base.year() + 1, 1, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(base.year(), base.month() + 1, 1).unwrap()
        };
        return Some(DateResolution {
            date: first_next_month - Duration::days(1),
            expression: "月底".to_string(),
            warning: None,
        });
    }

    if text.contains("月初") {
        let date = NaiveDate::from_ymd_opt(base.year(), base.month(), 1).unwrap();
        return Some(DateResolution {
            date,
            expression: "月初".to_string(),
            warning: past_warning(date, base),
        });
    }

    if let Some(day) = parse_next_month_day(text) {
        let (year, month) = if base.month() == 12 {
            (base.year() + 1, 1)
        } else {
            (base.year(), base.month() + 1)
        };
        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            return Some(DateResolution {
                date,
                expression: format!("下个月{}号", day),
                warning: None,
            });
        }
    }

    if text.contains("下周末") || text.contains("下星期末") || text.contains("下礼拜末")
    {
        let days_after_week_start = if names_sunday(text) { 13 } else { 12 };
        let date = week_start(base) + Duration::days(days_after_week_start);
        return Some(DateResolution {
            date,
            expression: "下周末".to_string(),
            warning: None,
        });
    }

    if let Some((week_offset, weekday, expression)) = parse_weekday_expression(text) {
        let date =
            week_start(base) + Duration::days((week_offset * 7 + weekday_index(weekday)) as i64);
        return Some(DateResolution {
            date,
            expression,
            warning: past_warning(date, base),
        });
    }

    if let Some((month, day, expression)) = parse_month_day(text) {
        if let Some(date) = NaiveDate::from_ymd_opt(base.year(), month, day) {
            return Some(DateResolution {
                date,
                expression,
                warning: past_warning(date, base),
            });
        }
    }

    None
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

fn weekday_index(day: Weekday) -> i32 {
    match day {
        Weekday::Mon => 0,
        Weekday::Tue => 1,
        Weekday::Wed => 2,
        Weekday::Thu => 3,
        Weekday::Fri => 4,
        Weekday::Sat => 5,
        Weekday::Sun => 6,
    }
}

fn past_warning(date: NaiveDate, base: NaiveDate) -> Option<String> {
    if date < base {
        Some(format!("日期 {} 已过去，请确认", date))
    } else {
        None
    }
}

fn parse_weekday_expression(text: &str) -> Option<(i32, Weekday, String)> {
    let weekdays = [
        ("一", Weekday::Mon),
        ("二", Weekday::Tue),
        ("三", Weekday::Wed),
        ("四", Weekday::Thu),
        ("五", Weekday::Fri),
        ("六", Weekday::Sat),
        ("日", Weekday::Sun),
        ("天", Weekday::Sun),
    ];

    for (cn, day) in weekdays {
        for prefix in ["周", "星期", "礼拜"] {
            let bare = format!("{prefix}{cn}");
            if text.contains(&format!("下下{bare}")) {
                return Some((2, day, format!("下下{bare}")));
            }
            if text.contains(&format!("下{bare}")) {
                return Some((1, day, format!("下{bare}")));
            }
            if text.contains(&format!("本{bare}"))
                || text.contains(&format!("这{bare}"))
                || text.contains(&format!("这个{bare}"))
            {
                return Some((0, day, bare));
            }
            if text.contains(&bare) {
                return Some((0, day, bare));
            }
        }
    }

    None
}

fn names_sunday(text: &str) -> bool {
    ["周日", "周天", "星期日", "星期天", "礼拜日", "礼拜天"]
        .iter()
        .any(|expr| text.contains(expr))
}

fn parse_next_month_day(text: &str) -> Option<u32> {
    if !text.contains("下个月") {
        return None;
    }

    for day in 1..=31 {
        if text.contains(&format!("下个月{}号", day)) || text.contains(&format!("下个月{}日", day))
        {
            return Some(day);
        }
    }

    for day in 1..=31 {
        if let Some(cn) = chinese_day(day) {
            if text.contains(&format!("下个月{cn}号")) || text.contains(&format!("下个月{cn}日"))
            {
                return Some(day);
            }
        }
    }

    None
}

fn parse_month_day(text: &str) -> Option<(u32, u32, String)> {
    for month in 1..=12 {
        for day in 1..=31 {
            let a = format!("{month}月{day}号");
            let b = format!("{month}月{day}日");
            if text.contains(&a) {
                return Some((month, day, a));
            }
            if text.contains(&b) {
                return Some((month, day, b));
            }
        }
    }

    None
}

fn chinese_day(day: u32) -> Option<String> {
    match day {
        1..=10 => Some(chinese_digit(day).to_string()),
        11..=19 => Some(format!("十{}", chinese_digit(day - 10))),
        20 => Some("二十".to_string()),
        21..=29 => Some(format!("二十{}", chinese_digit(day - 20))),
        30 => Some("三十".to_string()),
        31 => Some("三十一".to_string()),
        _ => None,
    }
}

fn chinese_digit(value: u32) -> &'static str {
    match value {
        1 => "一",
        2 => "二",
        3 => "三",
        4 => "四",
        5 => "五",
        6 => "六",
        7 => "七",
        8 => "八",
        9 => "九",
        10 => "十",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 5, 19).unwrap()
    }

    #[test]
    fn resolves_relative_days() {
        assert_eq!(
            resolve_date("明天拿文件", base()).unwrap().date,
            NaiveDate::from_ymd_opt(2026, 5, 20).unwrap()
        );
        assert_eq!(
            resolve_date("后天买牛奶", base()).unwrap().date,
            NaiveDate::from_ymd_opt(2026, 5, 21).unwrap()
        );
        assert_eq!(
            resolve_date("大后天检查备份", base()).unwrap().date,
            NaiveDate::from_ymd_opt(2026, 5, 22).unwrap()
        );
    }

    #[test]
    fn resolves_week_expressions_with_monday_start() {
        assert_eq!(
            resolve_date("这个周三交材料", base()).unwrap().date,
            NaiveDate::from_ymd_opt(2026, 5, 20).unwrap()
        );
        assert_eq!(
            resolve_date("本周日打电话", base()).unwrap().date,
            NaiveDate::from_ymd_opt(2026, 5, 24).unwrap()
        );
        assert_eq!(
            resolve_date("下周二复查", base()).unwrap().date,
            NaiveDate::from_ymd_opt(2026, 5, 26).unwrap()
        );
        assert_eq!(
            resolve_date("下星期二确认合同", base()).unwrap().date,
            NaiveDate::from_ymd_opt(2026, 5, 26).unwrap()
        );
        assert_eq!(
            resolve_date("下下周一续费", base()).unwrap().date,
            NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()
        );
    }

    #[test]
    fn resolves_bare_weekday_as_current_week_and_warns_when_past_or_today() {
        let result = resolve_date("周二寄快递", base()).unwrap();
        assert_eq!(result.date, NaiveDate::from_ymd_opt(2026, 5, 19).unwrap());
        assert_eq!(result.warning, None);

        let past = resolve_date("周一补发票", base()).unwrap();
        assert_eq!(past.date, NaiveDate::from_ymd_opt(2026, 5, 18).unwrap());
        assert!(past.warning.unwrap().contains("已过去"));
    }

    #[test]
    fn resolves_month_expressions() {
        assert_eq!(
            resolve_date("月底整理发票", base()).unwrap().date,
            NaiveDate::from_ymd_opt(2026, 5, 31).unwrap()
        );
        assert_eq!(
            resolve_date("月初检查账单", base()).unwrap().date,
            NaiveDate::from_ymd_opt(2026, 5, 1).unwrap()
        );
        assert_eq!(
            resolve_date("下个月3号还书", base()).unwrap().date,
            NaiveDate::from_ymd_opt(2026, 6, 3).unwrap()
        );
        assert_eq!(
            resolve_date("下个月三号还书", base()).unwrap().date,
            NaiveDate::from_ymd_opt(2026, 6, 3).unwrap()
        );
        assert_eq!(
            resolve_date("6月1号看活动", base()).unwrap().date,
            NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()
        );
    }

    #[test]
    fn resolves_before_friday_as_friday_this_week() {
        assert_eq!(
            resolve_date("周五之前提醒我提交申请", base()).unwrap().date,
            NaiveDate::from_ymd_opt(2026, 5, 22).unwrap()
        );
    }

    #[test]
    fn resolves_next_weekend_as_saturday() {
        assert_eq!(
            resolve_date("下周末约朋友吃饭", base()).unwrap().date,
            NaiveDate::from_ymd_opt(2026, 5, 30).unwrap()
        );
    }

    #[test]
    fn resolves_next_weekend_explicit_sunday() {
        assert_eq!(
            resolve_date("下周末周日约朋友吃饭", base()).unwrap().date,
            NaiveDate::from_ymd_opt(2026, 5, 31).unwrap()
        );
        assert_eq!(
            resolve_date("下周末星期日约朋友吃饭", base()).unwrap().date,
            NaiveDate::from_ymd_opt(2026, 5, 31).unwrap()
        );
    }

    #[test]
    fn resolves_explicit_past_month_day_with_warning() {
        let result = resolve_date("5月1号提醒我补发票", base()).unwrap();
        assert_eq!(result.date, NaiveDate::from_ymd_opt(2026, 5, 1).unwrap());
        assert!(result.warning.unwrap().contains("已过去"));
    }
}
