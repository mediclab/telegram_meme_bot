use chrono::{DateTime, Datelike, Days, TimeZone, Timelike, Utc};
use now::DateTimeNow;
use rand::seq::SliceRandom;
use teloxide::types::User;

pub fn get_user_text(user: &User) -> String {
    match &user.username {
        Some(uname) => format!("@{uname}"),
        None => format!(
            "<a href=\"tg://user?id={}\">{}</a>",
            user.id.0, user.first_name
        ),
    }
}

pub fn from_binary_to_hex(s: &str) -> String {
    s.chars()
        .collect::<Vec<char>>()
        .chunks(4)
        .map(|c| match c.iter().collect::<String>().as_str() {
            "0000" => "0".to_string(),
            "0001" => "1".to_string(),
            "0010" => "2".to_string(),
            "0011" => "3".to_string(),
            "0100" => "4".to_string(),
            "0101" => "5".to_string(),
            "0110" => "6".to_string(),
            "0111" => "7".to_string(),
            "1000" => "8".to_string(),
            "1001" => "9".to_string(),
            "1010" => "A".to_string(),
            "1011" => "B".to_string(),
            "1100" => "C".to_string(),
            "1101" => "D".to_string(),
            "1110" => "E".to_string(),
            "1111" => "F".to_string(),
            _ => String::new(),
        })
        .collect::<Vec<String>>()
        .join("")
}

pub fn from_hex_to_binary(s: &str) -> String {
    s.chars()
        .collect::<Vec<char>>()
        .iter()
        .map(|c| match c {
            '0' => "0000".to_string(),
            '1' => "0001".to_string(),
            '2' => "0010".to_string(),
            '3' => "0011".to_string(),
            '4' => "0100".to_string(),
            '5' => "0101".to_string(),
            '6' => "0110".to_string(),
            '7' => "0111".to_string(),
            '8' => "1000".to_string(),
            '9' => "1001".to_string(),
            'A' => "1010".to_string(),
            'B' => "1011".to_string(),
            'C' => "1100".to_string(),
            'D' => "1101".to_string(),
            'E' => "1110".to_string(),
            'F' => "1111".to_string(),
            _ => String::new(),
        })
        .collect::<Vec<String>>()
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
