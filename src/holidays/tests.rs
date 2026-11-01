use chrono::NaiveDate;

use super::{celebration_emoji_at, celebrations, random_celebration_emoji};

fn day(year: i32, month: u32, day_of_month: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day_of_month).expect("calendar date")
}

fn names_on(date: NaiveDate) -> Vec<&'static str> {
    celebrations(date)
        .into_iter()
        .map(|holiday| holiday.name())
        .collect()
}

fn emoji_on(date: NaiveDate) -> Vec<&'static str> {
    celebrations(date)
        .into_iter()
        .flat_map(|holiday| holiday.emoji().iter().copied())
        .collect()
}

#[test]
fn prescribed_days_in_2026_match_the_celebration_table() {
    let cases = [
        (day(2026, 1, 1), "New Year’s Day"),
        (day(2026, 1, 19), "Birthday of Martin Luther King, Jr."),
        (day(2026, 2, 16), "Washington’s Birthday"),
        (day(2026, 5, 25), "Memorial Day"),
        (day(2026, 6, 19), "Juneteenth National Independence Day"),
        (day(2026, 7, 4), "Independence Day"),
        (day(2026, 9, 7), "Labor Day"),
        (day(2026, 10, 12), "Columbus Day"),
        (day(2026, 11, 11), "Veterans Day"),
        (day(2026, 4, 5), "Easter"),
    ];
    for (date, name) in cases {
        assert_eq!(names_on(date), vec![name], "{date}");
    }
}

#[test]
fn noted_windows_include_their_bounds_and_exclude_the_next_day() {
    let inside = [
        (day(2025, 12, 31), "New Year’s Day"),
        (day(2026, 1, 1), "New Year’s Day"),
        (day(2026, 10, 26), "Halloween"),
        (day(2026, 10, 31), "Halloween"),
        (day(2026, 11, 22), "Thanksgiving Day"),
        (day(2026, 11, 26), "Thanksgiving Day"),
        (day(2026, 11, 27), "Christmas Day"),
        (day(2026, 12, 25), "Christmas Day"),
    ];
    for (date, name) in inside {
        assert_eq!(names_on(date), vec![name], "{date}");
    }
    for date in [
        day(2026, 1, 2),
        day(2026, 12, 30),
        day(2026, 10, 25),
        day(2026, 11, 1),
        day(2026, 11, 21),
        day(2026, 12, 26),
        day(2026, 7, 3),
        day(2026, 8, 15),
    ] {
        assert!(names_on(date).is_empty(), "{date}");
    }
}

#[test]
fn inauguration_follows_the_sunday_rule_and_can_share_mlk_day() {
    assert_eq!(
        names_on(day(2025, 1, 20)),
        vec!["Birthday of Martin Luther King, Jr.", "Inauguration Day"]
    );
    assert_eq!(celebration_emoji_at(day(2025, 1, 20), 0), Some("🕊️"));
    assert_eq!(celebration_emoji_at(day(2025, 1, 20), 1), Some("🏛️"));
    assert_eq!(celebration_emoji_at(day(2025, 1, 20), 2), Some("🕊️"));
    assert_eq!(
        names_on(day(2013, 1, 21)),
        vec!["Birthday of Martin Luther King, Jr.", "Inauguration Day"]
    );
    assert!(
        names_on(day(2013, 1, 20)).is_empty(),
        "sunday moves the ceremony"
    );
    assert!(names_on(day(1965, 1, 20)).is_empty());
    assert_eq!(names_on(day(2021, 1, 20)), vec!["Inauguration Day"]);
    assert_eq!(names_on(day(2029, 1, 20)), vec!["Inauguration Day"]);
}

#[test]
fn western_easter_matches_usno_and_skips_orthodox_easter() {
    assert_eq!(names_on(day(2024, 3, 31)), vec!["Easter"]);
    assert_eq!(names_on(day(2025, 4, 20)), vec!["Easter"]);
    assert_eq!(names_on(day(2026, 4, 5)), vec!["Easter"]);
    assert_eq!(names_on(day(2027, 3, 28)), vec!["Easter"]);
    assert_eq!(names_on(day(2013, 3, 31)), vec!["Easter"]);
    assert!(
        names_on(day(2026, 4, 12)).is_empty(),
        "2026-04-12 is Orthodox Easter"
    );
    assert!(names_on(day(2026, 4, 4)).is_empty());
}

#[test]
fn emoji_lists_match_the_notes() {
    assert_eq!(emoji_on(day(2026, 1, 1)), vec!["🥂"]);
    assert_eq!(emoji_on(day(2026, 7, 4)), ["🎆", "🎇", "🇺🇸", "🗽", "🧨"]);
    assert_eq!(emoji_on(day(2026, 11, 26)), ["🦃", "🌽", "🥧", "🍂", "🏈"]);
    assert_eq!(emoji_on(day(2026, 12, 25)), ["🎄", "🎅", "🎁", "⛄", "❄️"]);
    assert_eq!(emoji_on(day(2026, 10, 31)), ["🎃", "👻", "🦇", "🕷️", "🍬"]);
    assert_eq!(emoji_on(day(2026, 4, 5)), ["🥚", "🐰", "🐣", "🌷", "🧺"]);
    assert_eq!(emoji_on(day(2026, 1, 19)), vec!["🕊️"]);
    assert_eq!(emoji_on(day(2026, 2, 16)), vec!["🦅"]);
    assert_eq!(emoji_on(day(2026, 5, 25)), vec!["🇺🇸"]);
    assert_eq!(emoji_on(day(2026, 6, 19)), vec!["✊"]);
    assert_eq!(emoji_on(day(2026, 9, 7)), vec!["🛠️"]);
    assert_eq!(emoji_on(day(2026, 10, 12)), vec!["⛵"]);
    assert_eq!(emoji_on(day(2026, 11, 11)), vec!["🎖️"]);
    assert_eq!(emoji_on(day(2029, 1, 20)), vec!["🏛️"]);
}

#[test]
fn random_emoji_is_one_of_the_day_s_emoji() {
    let new_year = day(2026, 1, 1);
    for _ in 0..8 {
        assert_eq!(random_celebration_emoji(new_year), Some("🥂"));
    }
    assert_eq!(random_celebration_emoji(day(2026, 8, 15)), None);
    assert_eq!(celebration_emoji_at(day(2026, 8, 15), 0), None);
    let halloween = day(2026, 10, 31);
    let allowed = ["🎃", "👻", "🦇", "🕷️", "🍬"];
    for _ in 0..12 {
        let emoji = random_celebration_emoji(halloween).expect("halloween emoji");
        assert!(allowed.contains(&emoji), "{emoji}");
    }
}
