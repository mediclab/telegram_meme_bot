use chrono::{DateTime, Datelike, Days, Timelike, Utc};
use now::DateTimeNow;
use teloxide::types::User;

pub fn get_user_text(user: &User) -> String {
    match &user.username {
        Some(uname) => format!("@{}", uname),
        None => format!("[{}](tg://user?id={})", user.first_name, user.id.0),
    }
}

pub fn last_workday_of_month() -> DateTime<Utc> {
    let mut date = Utc::now().end_of_month();

    date = match date.weekday().number_from_monday() {
        6 => date.checked_sub_days(Days::new(1)).unwrap(),
        7 => date.checked_sub_days(Days::new(2)).unwrap(),
        _ => date,
    };

    date.with_hour(18)
        .unwrap()
        .with_minute(59)
        .unwrap()
        .with_second(59)
        .unwrap()
}

pub fn first_workday_of_month() -> DateTime<Utc> {
    let mut date = Utc::now().beginning_of_month();

    date = match date.weekday().number_from_monday() {
        6 => date.checked_sub_days(Days::new(1)).unwrap(),
        7 => date.checked_sub_days(Days::new(2)).unwrap(),
        1 => date.checked_sub_days(Days::new(3)).unwrap(),
        _ => date,
    };

    date.with_hour(19)
        .unwrap()
        .with_minute(00)
        .unwrap()
        .with_second(00)
        .unwrap()
}

pub fn first_workday_of_week() -> DateTime<Utc> {
    Utc::now()
        .beginning_of_week()
        .checked_sub_days(Days::new(3))
        .unwrap()
        .with_hour(19)
        .unwrap()
        .with_minute(00)
        .unwrap()
        .with_second(00)
        .unwrap()
}

pub fn last_workday_of_week() -> DateTime<Utc> {
    Utc::now()
        .end_of_week()
        .checked_sub_days(Days::new(2))
        .unwrap()
        .with_hour(18)
        .unwrap()
        .with_minute(59)
        .unwrap()
        .with_second(59)
        .unwrap()
}

pub fn last_workday_of_year() -> DateTime<Utc> {
    let mut date = Utc::now().end_of_year();

    date = match date.weekday().number_from_monday() {
        6 => date.checked_sub_days(Days::new(1)).unwrap(),
        7 => date.checked_sub_days(Days::new(2)).unwrap(),
        _ => date,
    };

    date.with_hour(18)
        .unwrap()
        .with_minute(59)
        .unwrap()
        .with_second(59)
        .unwrap()
}

pub fn first_workday_of_year() -> DateTime<Utc> {
    let mut date = Utc::now().beginning_of_year();

    date = match date.weekday().number_from_monday() {
        6 => date.checked_sub_days(Days::new(1)).unwrap(),
        7 => date.checked_sub_days(Days::new(2)).unwrap(),
        1 => date.checked_sub_days(Days::new(3)).unwrap(),
        _ => date,
    };

    date.with_hour(19)
        .unwrap()
        .with_minute(00)
        .unwrap()
        .with_second(00)
        .unwrap()
}
