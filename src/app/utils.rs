use chrono::{DateTime, Datelike, Duration, TimeZone, Timelike, Utc, Weekday};
use itertools::Itertools;
use now::DateTimeNow;
use std::str;
use teloxide::types::User;

pub fn get_user_text(user: &User) -> String {
    match &user.username {
        Some(uname) => format!("@{uname}"),
        None => format!("<a href=\"{}\">{}</a>", user.url(), user.first_name),
    }
}

pub fn from_binary_to_hex(s: &str) -> String {
    s.as_bytes()
        .chunks(4)
        .map(|c| {
            format!(
                "{:X}",
                i8::from_str_radix(str::from_utf8(c).expect("Wrong char"), 2).unwrap()
            )
        })
        .join("")
}

pub fn from_hex_to_binary(s: &str) -> String {
    s.chars()
        .map(|c| format!("{:04b}", i8::from_str_radix(&c.to_string(), 16).unwrap()))
        .join("")
}

pub struct Messages {}

impl Messages {
    pub fn pluralize(num: i64, texts: (&str, &str, &str)) -> String {
        let last = num % 100;

        let plural = if (11..=19).contains(&last) {
            texts.2.to_string()
        } else {
            match last % 10 {
                1 => texts.0.to_string(),
                2..=4 => texts.1.to_string(),
                _ => texts.2.to_string(),
            }
        };

        format!("{num} {plural}")
    }
}

#[derive(PartialEq)]
pub enum Period {
    Week,
    Month,
    Year,
    Custom { from: DateTime<Utc>, to: DateTime<Utc> },
}

impl Period {
    pub fn dates(&self) -> (DateTime<Utc>, DateTime<Utc>) {
        match *self {
            Period::Week => self.week_dates(),
            Period::Month => self.month_dates(),
            Period::Year => self.year_dates(),
            Period::Custom { from, to } => (from, to),
        }
    }

    pub fn is_today_a_friday() -> bool {
        Weekday::Fri == Utc::now().weekday()
    }

    pub fn is_today_a_last_month_day() -> bool {
        let now = Utc::now();

        if (now.month() != (now + Duration::try_days(3).unwrap()).month()) && Period::is_today_a_friday() {
            return true;
        }

        (now.end_of_month().day() == now.day()) && (Weekday::Sun != now.weekday()) && (Weekday::Sat != now.weekday())
    }

    pub fn is_today_a_last_year_day() -> bool {
        let now = Utc::now();

        now.end_of_year().month() == now.month() && now.end_of_year().day() == now.day()
    }

    fn from(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> (DateTime<Utc>, DateTime<Utc>) {
        (
            Utc.with_ymd_and_hms(start.year(), start.month(), start.day(), 16, 0, 0)
                .unwrap(),
            Utc.with_ymd_and_hms(end.year(), end.month(), end.day(), 15, 59, 59)
                .unwrap()
                .with_nanosecond(999999999)
                .unwrap(),
        )
    }

    fn week_dates(&self) -> (DateTime<Utc>, DateTime<Utc>) {
        let start_week = Utc::now().beginning_of_week() + Duration::try_days(-3).unwrap();
        let end_week = Utc::now().end_of_week() + Duration::try_days(-2).unwrap();

        self.from(start_week, end_week)
    }

    fn month_dates(&self) -> (DateTime<Utc>, DateTime<Utc>) {
        let start_month = self.get_first_work_day(&Utc::now().beginning_of_month());
        let end_month = self.get_last_work_day(&Utc::now().end_of_month());

        self.from(start_month, end_month)
    }

    fn year_dates(&self) -> (DateTime<Utc>, DateTime<Utc>) {
        let start_year = self.get_first_work_day(&Utc::now().beginning_of_year());
        let end_year = self.get_last_work_day(&Utc::now().end_of_year());

        self.from(start_year, end_year)
    }

    fn get_last_work_day(&self, date: &DateTime<Utc>) -> DateTime<Utc> {
        match date.weekday() {
            Weekday::Sat => *date + Duration::try_days(-1).unwrap(),
            Weekday::Sun => *date + Duration::try_days(-2).unwrap(),
            _ => *date,
        }
    }

    fn get_first_work_day(&self, date: &DateTime<Utc>) -> DateTime<Utc> {
        match date.weekday() {
            Weekday::Sat => *date + Duration::try_days(-1).unwrap(),
            Weekday::Sun => *date + Duration::try_days(-2).unwrap(),
            Weekday::Mon => *date + Duration::try_days(-3).unwrap(),
            _ => *date,
        }
    }
}
