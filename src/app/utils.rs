use chrono::{DateTime, Datelike, Days, TimeZone, Timelike, Utc};
use itertools::Itertools;
use now::DateTimeNow;
use rand::seq::SliceRandom;
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

pub struct Messages {
    messages: Vec<String>,
}

impl Messages {
    pub fn load(text: &str) -> Self {
        let vec: Vec<&str> = text
            .split(';')
            .filter(|text| !text.trim().is_empty())
            .collect();

        Self {
            messages: vec.iter().map(|s| s.trim().to_string()).collect(),
        }
    }

    pub fn random(&self) -> &String {
        self.messages.choose(&mut rand::thread_rng()).unwrap()
    }

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
}

impl Period {
    pub fn dates(&self) -> (DateTime<Utc>, DateTime<Utc>) {
        match *self {
            Period::Week => Period::week_dates(),
            Period::Month => Period::month_dates(),
            Period::Year => Period::year_dates(),
        }
    }

    pub fn is_today_a_friday() -> bool {
        chrono::Weekday::Fri == Utc::now().weekday()
    }

    pub fn is_today_a_last_month_day() -> bool {
        Utc::now().end_of_month().day() == Utc::now().day()
    }

    pub fn is_today_a_last_year_day() -> bool {
        let now = Utc::now();

        now.end_of_year().month() == now.month() && now.end_of_year().day() == now.day()
    }

    fn week_dates() -> (DateTime<Utc>, DateTime<Utc>) {
        let start_week = Utc::now()
            .beginning_of_week()
            .checked_sub_days(Days::new(3))
            .unwrap();

        let start = Utc
            .with_ymd_and_hms(
                start_week.year(),
                start_week.month(),
                start_week.day(),
                16,
                0,
                0,
            )
            .unwrap();

        let end_week = Utc::now()
            .end_of_week()
            .checked_sub_days(Days::new(2))
            .unwrap();

        let end = Utc
            .with_ymd_and_hms(
                end_week.year(),
                end_week.month(),
                end_week.day(),
                15,
                59,
                59,
            )
            .unwrap()
            .with_nanosecond(999999);

        (start, end.unwrap())
    }

    fn month_dates() -> (DateTime<Utc>, DateTime<Utc>) {
        let start_month = Period::get_first_work_day(&Utc::now().beginning_of_month());

        let start = Utc
            .with_ymd_and_hms(
                start_month.year(),
                start_month.month(),
                start_month.day(),
                16,
                0,
                0,
            )
            .unwrap();

        let end_month = Period::get_last_work_day(&Utc::now().end_of_month());

        let end = Utc
            .with_ymd_and_hms(
                end_month.year(),
                end_month.month(),
                end_month.day(),
                15,
                59,
                59,
            )
            .unwrap()
            .with_nanosecond(999999);

        (start, end.unwrap())
    }

    fn year_dates() -> (DateTime<Utc>, DateTime<Utc>) {
        let start_month = Period::get_first_work_day(&Utc::now().beginning_of_year());

        let start = Utc
            .with_ymd_and_hms(
                start_month.year(),
                start_month.month(),
                start_month.day(),
                16,
                0,
                0,
            )
            .unwrap();

        let end_month = Period::get_last_work_day(&Utc::now().end_of_year());

        let end = Utc
            .with_ymd_and_hms(
                end_month.year(),
                end_month.month(),
                end_month.day(),
                15,
                59,
                59,
            )
            .unwrap()
            .with_nanosecond(999999);

        (start, end.unwrap())
    }

    fn get_last_work_day(date: &DateTime<Utc>) -> DateTime<Utc> {
        match date.weekday().number_from_monday() {
            6 => date.checked_sub_days(Days::new(1)).unwrap(),
            7 => date.checked_sub_days(Days::new(2)).unwrap(),
            _ => *date,
        }
    }

    fn get_first_work_day(date: &DateTime<Utc>) -> DateTime<Utc> {
        match date.weekday().number_from_monday() {
            6 => date.checked_sub_days(Days::new(1)).unwrap(),
            7 => date.checked_sub_days(Days::new(2)).unwrap(),
            1 => date.checked_sub_days(Days::new(3)).unwrap(),
            _ => *date,
        }
    }
}
