use chrono::{DateTime, Datelike, Duration, NaiveTime, Utc};

pub fn next_weekly_occurrence(
    now: DateTime<Utc>,
    day_of_week: u8,
    hour: u8,
    minute: u8,
) -> Option<DateTime<Utc>> {
    if !(1..=7).contains(&day_of_week) || hour > 23 || minute > 59 {
        return None;
    }

    let current_weekday = now.weekday().number_from_monday() as i64;
    let target_weekday = day_of_week as i64;
    let mut days_ahead = (target_weekday - current_weekday + 7) % 7;

    let time = NaiveTime::from_hms_opt(hour as u32, minute as u32, 0)?;
    let mut target_date = now.date_naive() + Duration::days(days_ahead);
    let mut target_naive = target_date.and_time(time);

    if days_ahead == 0 && target_naive <= now.naive_utc() {
        days_ahead = 7;
        target_date = now.date_naive() + Duration::days(days_ahead);
        target_naive = target_date.and_time(time);
    }

    Some(DateTime::from_naive_utc_and_offset(target_naive, Utc))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn returns_same_week_when_target_is_later_today() {
        let now = Utc.with_ymd_and_hms(2026, 4, 13, 10, 0, 0).unwrap(); // Monday
        let next = next_weekly_occurrence(now, 1, 11, 0).unwrap();
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 4, 13, 11, 0, 0).unwrap());
    }

    #[test]
    fn rolls_to_next_week_when_target_time_has_passed() {
        let now = Utc.with_ymd_and_hms(2026, 4, 13, 10, 0, 0).unwrap(); // Monday
        let next = next_weekly_occurrence(now, 1, 9, 30).unwrap();
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 4, 20, 9, 30, 0).unwrap());
    }

    #[test]
    fn computes_next_day_in_same_week() {
        let now = Utc.with_ymd_and_hms(2026, 4, 13, 10, 0, 0).unwrap(); // Monday
        let next = next_weekly_occurrence(now, 3, 14, 45).unwrap(); // Wednesday
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 4, 15, 14, 45, 0).unwrap());
    }

    #[test]
    fn rejects_invalid_inputs() {
        let now = Utc.with_ymd_and_hms(2026, 4, 13, 10, 0, 0).unwrap();
        assert!(next_weekly_occurrence(now, 0, 10, 10).is_none());
        assert!(next_weekly_occurrence(now, 3, 24, 10).is_none());
        assert!(next_weekly_occurrence(now, 3, 23, 60).is_none());
    }
}
