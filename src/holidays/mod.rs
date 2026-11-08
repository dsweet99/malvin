mod calendar;

use chrono::{Datelike, Days, NaiveDate, Weekday};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Celebration {
    name: &'static str,
    emoji: &'static [&'static str],
}

impl Celebration {
    fn new(name: &'static str, emoji: &'static [&'static str]) -> Self {
        assert!(!emoji.is_empty(), "a holiday needs at least one emoji");
        Self { name, emoji }
    }

    #[allow(dead_code)]
    #[must_use]
    pub(crate) const fn name(self) -> &'static str {
        self.name
    }

    #[must_use]
    pub(crate) const fn emoji(self) -> &'static [&'static str] {
        self.emoji
    }
}

struct FixedHoliday {
    month: u32,
    day_lo: u32,
    day_hi: u32,
    name: &'static str,
    emoji: &'static [&'static str],
}

struct WeekdayHoliday {
    month: u32,
    weekday: Weekday,
    n: u8,
    name: &'static str,
    emoji: &'static [&'static str],
}

const FIXED: &[FixedHoliday] = &[
    FixedHoliday {
        month: 1,
        day_lo: 1,
        day_hi: 1,
        name: "New Year’s Day",
        emoji: &["🥂"],
    },
    FixedHoliday {
        month: 12,
        day_lo: 31,
        day_hi: 31,
        name: "New Year’s Day",
        emoji: &["🥂"],
    },
    FixedHoliday {
        month: 6,
        day_lo: 19,
        day_hi: 19,
        name: "Juneteenth National Independence Day",
        emoji: &["✊"],
    },
    FixedHoliday {
        month: 7,
        day_lo: 4,
        day_hi: 4,
        name: "Independence Day",
        emoji: &["🎆", "🎇", "🇺🇸", "🗽", "🧨"],
    },
    FixedHoliday {
        month: 11,
        day_lo: 11,
        day_hi: 11,
        name: "Veterans Day",
        emoji: &["🎖️"],
    },
    FixedHoliday {
        month: 10,
        day_lo: 26,
        day_hi: 31,
        name: "Halloween",
        emoji: &["🎃", "👻", "🦇", "🕷️", "🍬"],
    },
];

const WEEKDAY: &[WeekdayHoliday] = &[
    WeekdayHoliday {
        month: 1,
        weekday: Weekday::Mon,
        n: 3,
        name: "Birthday of Martin Luther King, Jr.",
        emoji: &["🕊️"],
    },
    WeekdayHoliday {
        month: 2,
        weekday: Weekday::Mon,
        n: 3,
        name: "Washington’s Birthday",
        emoji: &["🦅"],
    },
    WeekdayHoliday {
        month: 5,
        weekday: Weekday::Mon,
        n: calendar::LAST_WEEKDAY,
        name: "Memorial Day",
        emoji: &["🇺🇸"],
    },
    WeekdayHoliday {
        month: 9,
        weekday: Weekday::Mon,
        n: 1,
        name: "Labor Day",
        emoji: &["🛠️"],
    },
    WeekdayHoliday {
        month: 10,
        weekday: Weekday::Mon,
        n: 2,
        name: "Columbus Day",
        emoji: &["⛵"],
    },
];

const THANKSGIVING_EMOJI: &[&str] = &["🦃", "🌽", "🥧", "🍂", "🏈"];
const CHRISTMAS_EMOJI: &[&str] = &["🎄", "🎅", "🎁", "⛄", "❄️"];
const INAUGURATION_EMOJI: &[&str] = &["🏛️"];
const EASTER_EMOJI: &[&str] = &["🥚", "🐰", "🐣", "🌷", "🧺"];

#[must_use]
pub(crate) fn celebrations(date: NaiveDate) -> Vec<Celebration> {
    let mut found = Vec::new();
    push_fixed(date, &mut found);
    push_weekday(date, &mut found);
    push_thanksgiving(date, &mut found);
    push_christmas(date, &mut found);
    push_inauguration(date, &mut found);
    push_easter(date, &mut found);
    found
}

#[must_use]
pub(crate) fn random_celebration_emoji(date: NaiveDate) -> Option<&'static str> {
    let len = emoji_pool(date).len();
    if len == 0 {
        None
    } else {
        celebration_emoji_at(date, fastrand::usize(..len))
    }
}

#[must_use]
pub(crate) fn celebration_emoji_at(date: NaiveDate, index: usize) -> Option<&'static str> {
    emoji_at(&emoji_pool(date), index)
}

fn emoji_pool(date: NaiveDate) -> Vec<&'static str> {
    celebrations(date)
        .into_iter()
        .flat_map(|holiday| holiday.emoji().iter().copied())
        .collect()
}

fn emoji_at<'a>(pool: &[&'a str], index: usize) -> Option<&'a str> {
    if pool.is_empty() {
        None
    } else {
        Some(pool[index % pool.len()])
    }
}

fn push_fixed(date: NaiveDate, out: &mut Vec<Celebration>) {
    for spec in FIXED {
        if date.month() == spec.month && (spec.day_lo..=spec.day_hi).contains(&date.day()) {
            out.push(Celebration::new(spec.name, spec.emoji));
        }
    }
}

fn push_weekday(date: NaiveDate, out: &mut Vec<Celebration>) {
    for spec in WEEKDAY {
        if date == calendar::weekday_holiday(date.year(), spec.month, spec.weekday, spec.n) {
            out.push(Celebration::new(spec.name, spec.emoji));
        }
    }
}

fn push_thanksgiving(date: NaiveDate, out: &mut Vec<Celebration>) {
    let thursday = calendar::thanksgiving(date.year());
    let sunday = thursday
        .checked_sub_days(Days::new(4))
        .expect("Sunday before Thanksgiving");
    if date >= sunday && date <= thursday {
        out.push(Celebration::new("Thanksgiving Day", THANKSGIVING_EMOJI));
    }
}

fn push_christmas(date: NaiveDate, out: &mut Vec<Celebration>) {
    let friday = calendar::thanksgiving(date.year())
        .checked_add_days(Days::new(1))
        .expect("day after Thanksgiving");
    let christmas = NaiveDate::from_ymd_opt(date.year(), 12, 25).expect("December 25");
    if date >= friday && date <= christmas {
        out.push(Celebration::new("Christmas Day", CHRISTMAS_EMOJI));
    }
}

fn push_inauguration(date: NaiveDate, out: &mut Vec<Celebration>) {
    if calendar::inauguration_date(date.year()) == Some(date) {
        out.push(Celebration::new("Inauguration Day", INAUGURATION_EMOJI));
    }
}

fn push_easter(date: NaiveDate, out: &mut Vec<Celebration>) {
    if date == calendar::western_easter(date.year()) {
        out.push(Celebration::new("Easter", EASTER_EMOJI));
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod holidays_tests;
