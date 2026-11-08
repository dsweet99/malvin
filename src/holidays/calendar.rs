use chrono::{Datelike, Days, NaiveDate, Weekday};

pub(super) const LAST_WEEKDAY: u8 = 0;

pub(super) fn weekday_holiday(year: i32, month: u32, weekday: Weekday, n: u8) -> NaiveDate {
    if n == LAST_WEEKDAY {
        last_weekday(year, month, weekday)
    } else {
        nth_weekday(year, month, weekday, n)
    }
}

pub(super) fn thanksgiving(year: i32) -> NaiveDate {
    nth_weekday(year, 11, Weekday::Thu, 4)
}

pub(super) fn inauguration_date(year: i32) -> Option<NaiveDate> {
    if year < 1969 || year.rem_euclid(4) != 1 {
        return None;
    }
    let jan20 = NaiveDate::from_ymd_opt(year, 1, 20).expect("January 20");
    if jan20.weekday() == Weekday::Sun {
        Some(jan20.checked_add_days(Days::new(1)).expect("January 21"))
    } else {
        Some(jan20)
    }
}

pub(super) fn western_easter(year: i32) -> NaiveDate {
    let golden = year.rem_euclid(19);
    let century = year / 100;
    let year_mod = year.rem_euclid(100);
    let lunar = (19 * golden + century - century / 4 - (century - (century + 8) / 25 + 1) / 3 + 15)
        .rem_euclid(30);
    let weekday_shift =
        (32 + 2 * century.rem_euclid(4) + 2 * (year_mod / 4) - lunar - year_mod.rem_euclid(4))
            .rem_euclid(7);
    let correction = (golden + 11 * lunar + 22 * weekday_shift) / 451;
    let month_day = lunar + weekday_shift - 7 * correction + 114;
    let month = u32::try_from(month_day / 31).expect("easter month");
    let day = u32::try_from(month_day.rem_euclid(31) + 1).expect("easter day");
    NaiveDate::from_ymd_opt(year, month, day).expect("western easter")
}

fn nth_weekday(year: i32, month: u32, weekday: Weekday, n: u8) -> NaiveDate {
    let first = NaiveDate::from_ymd_opt(year, month, 1).expect("first of month");
    let shift = (7 + weekday.num_days_from_monday() - first.weekday().num_days_from_monday()) % 7;
    let day = 1 + shift + u32::from(n - 1) * 7;
    NaiveDate::from_ymd_opt(year, month, day).expect("nth weekday")
}

fn last_weekday(year: i32, month: u32, weekday: Weekday) -> NaiveDate {
    let next_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .expect("first of next month");
    let last = next_month
        .checked_sub_days(Days::new(1))
        .expect("last day of month");
    let back = (7 + last.weekday().num_days_from_monday() - weekday.num_days_from_monday()) % 7;
    last.checked_sub_days(Days::new(u64::from(back)))
        .expect("weekday in month")
}
