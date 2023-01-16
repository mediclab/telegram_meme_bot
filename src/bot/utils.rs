use chrono::{DateTime, Datelike, Days, TimeZone, Utc};
use now::DateTimeNow;
use teloxide::types::User;

pub fn get_user_text(user: &User) -> String {
    match &user.username {
        Some(uname) => format!("@{}", uname),
        None => format!("[{}](tg://user?id={})", user.first_name, user.id.0),
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
            .beginning_of_week()
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
            .unwrap();

        (start, end)
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
            .unwrap();

        (start, end)
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
            .unwrap();

        (start, end)
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
