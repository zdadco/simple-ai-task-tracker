use chrono::{Datelike, Local, NaiveTime, TimeZone};

use crate::db::digests::DigestKind;

/// Inclusive start, exclusive end — unix timestamps in local TZ interpretation.
#[derive(Debug, Clone, Copy)]
pub struct PeriodBounds {
    pub start: i64,
    pub end: i64,
}

pub fn period_bounds(kind: DigestKind, now: chrono::DateTime<Local>) -> PeriodBounds {
    let date = now.date_naive();
    match kind {
        DigestKind::Daily => {
            let start = date
                .and_hms_opt(0, 0, 0)
                .unwrap();
            let end = (date + chrono::Duration::days(1))
                .and_hms_opt(0, 0, 0)
                .unwrap();
            PeriodBounds {
                start: Local.from_local_datetime(&start).unwrap().timestamp(),
                end: Local.from_local_datetime(&end).unwrap().timestamp(),
            }
        }
        DigestKind::Weekly => {
            let weekday = date.weekday().num_days_from_monday() as i64;
            let monday = date - chrono::Duration::days(weekday);
            let next_monday = monday + chrono::Duration::days(7);
            let start = monday.and_hms_opt(0, 0, 0).unwrap();
            let end = next_monday.and_hms_opt(0, 0, 0).unwrap();
            PeriodBounds {
                start: Local.from_local_datetime(&start).unwrap().timestamp(),
                end: Local.from_local_datetime(&end).unwrap().timestamp(),
            }
        }
        DigestKind::Monthly => {
            let start_date = chrono::NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap();
            let end_date = if date.month() == 12 {
                chrono::NaiveDate::from_ymd_opt(date.year() + 1, 1, 1).unwrap()
            } else {
                chrono::NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1).unwrap()
            };
            let start = start_date.and_hms_opt(0, 0, 0).unwrap();
            let end = end_date.and_hms_opt(0, 0, 0).unwrap();
            PeriodBounds {
                start: Local.from_local_datetime(&start).unwrap().timestamp(),
                end: Local.from_local_datetime(&end).unwrap().timestamp(),
            }
        }
    }
}

/// Scheduled fire time for the current period (local).
pub fn fire_datetime(
    kind: DigestKind,
    now: chrono::DateTime<Local>,
    hhmm: &str,
) -> Option<chrono::DateTime<Local>> {
    let (hour, minute) = parse_hhmm(hhmm)?;
    let bounds = period_bounds(kind, now);
    let start_dt = Local.timestamp_opt(bounds.start, 0).single()?;
    let date = start_dt.date_naive();
    let naive = date.and_time(NaiveTime::from_hms_opt(hour, minute, 0)?);
    Local.from_local_datetime(&naive).single()
}

pub fn parse_hhmm(s: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = s.trim().split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    let hour: u32 = parts[0].parse().ok()?;
    let minute: u32 = parts[1].parse().ok()?;
    if hour > 23 || minute > 59 {
        return None;
    }
    Some((hour, minute))
}

pub fn should_fire(kind: DigestKind, now: chrono::DateTime<Local>, hhmm: &str) -> bool {
    let Some(fire_at) = fire_datetime(kind, now, hhmm) else {
        return false;
    };
    now >= fire_at
}

#[allow(dead_code)]
pub fn format_period_label(start: i64, end: i64) -> String {
    let s = Local.timestamp_opt(start, 0).single();
    let e = Local.timestamp_opt(end, 0).single();
    match (s, e) {
        (Some(s), Some(e)) => format!(
            "{} — {}",
            s.format("%Y-%m-%d %H:%M"),
            e.format("%Y-%m-%d %H:%M")
        ),
        _ => format!("{start} — {end}"),
    }
}
