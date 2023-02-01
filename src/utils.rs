use crate::database::models::AddUser;
use crate::Application;
use chrono::{DateTime, Datelike, Days, TimeZone, Timelike, Utc};
use now::DateTimeNow;
use opencv::{
    core::{Mat, Size},
    imgcodecs,
    imgcodecs::ImreadModes,
    imgproc,
    imgproc::InterpolationFlags,
    prelude::*,
};

use teloxide::{
    net::Download,
    prelude::*,
    types::{PhotoSize, User},
    Bot,
};

use rand::seq::SliceRandom;
use std::error::Error;
use std::thread::sleep;
use std::time::Duration;
use tokio::fs::File;

pub fn get_user_text(user: &User) -> String {
    match &user.username {
        Some(uname) => format!("@{uname}"),
        None => format!("[{}](tg://user?id={})", user.first_name, user.id.0),
    }
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

#[derive(Clone)]
pub struct ImageHash {
    image: Mat,
}

impl ImageHash {
    pub fn new(filename: &str) -> Self {
        Self {
            image: imgcodecs::imread(filename, ImreadModes::IMREAD_COLOR as i32)
                .unwrap_or_default(),
        }
    }

    pub fn grayscale(mut self) -> Self {
        let mut gray = Mat::default();
        imgproc::cvt_color(&self.image, &mut gray, imgproc::COLOR_BGR2GRAY, 0).unwrap_or_default();

        self.image = gray;
        self
    }

    pub fn resize(mut self, hash_size: i32) -> Self {
        let mut resized = Mat::default();
        imgproc::resize(
            &self.image,
            &mut resized,
            Size::new(hash_size, hash_size),
            0.0,
            0.0,
            InterpolationFlags::INTER_AREA as i32,
        )
        .unwrap_or_default();

        self.image = resized;
        self
    }

    pub fn threshold(mut self) -> Self {
        let mean = opencv::core::mean(&self.image, &Mat::default()).expect("Can't mean");
        let mut t_image = Mat::default();
        imgproc::threshold(&self.image, &mut t_image, mean.0[0], 255.0, 0).unwrap_or_default();

        self.image = t_image;
        self
    }

    pub fn hash(&self) -> Option<String> {
        let a_image = self.image.to_vec_2d::<u8>();

        if a_image.is_err() {
            return None;
        }

        let hash = a_image
            .unwrap()
            .iter()
            .map(|row| {
                row.iter()
                    .map(|item| {
                        if *item == 255 {
                            String::from("1")
                        } else {
                            String::from("0")
                        }
                    })
                    .collect::<Vec<String>>()
                    .join("")
            })
            .collect::<Vec<String>>()
            .join("");

        Some(hash)
    }
}

pub async fn generate_hashes(
    bot: &Bot,
    file_id: &String,
) -> Result<(Option<String>, Option<String>), Box<dyn Error + Send + Sync>> {
    let photo = bot.get_file(file_id).await?;
    let path = format!("/tmp/{}", uuid::Uuid::new_v4());
    let mut file = File::create(&path).await?;

    bot.download_file(&photo.path, &mut file).await?;

    sleep(Duration::from_millis(50)); // Sometimes downloading is very fast
    debug!("Filesize {path} is = {}", std::fs::metadata(&path)?.len());

    let cv_image = ImageHash::new(&path).grayscale();
    let hash = cv_image.clone().resize(32).threshold().hash();
    let hash_min = cv_image.resize(4).threshold().hash();

    std::fs::remove_file(&path).unwrap_or_default();

    if hash.is_none() || hash_min.is_none() {
        return Err("Error in opencv hashing")?;
    }

    Ok((
        Some(from_binary_to_hex(&hash.unwrap())),
        Some(from_binary_to_hex(&hash_min.unwrap())),
    ))
}

pub async fn update_hashes(
    bot: &Bot,
    app: &Application,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let memes = app.database.get_memes_without_hashes()?;

    info!("Count updating memes hashes = {}", memes.len());

    for meme in &memes {
        info!("Start updating hashes for = {}", &meme.uuid);
        let json: Vec<PhotoSize> = match serde_json::from_value(meme.photos.clone().unwrap()) {
            Ok(res) => res,
            Err(_) => {
                error!("Can't deserialize photos of meme = {}", &meme.uuid);
                continue;
            }
        };

        let (hash, hash_min) = match generate_hashes(bot, &json[0].file.id).await {
            Ok(res) => res,
            Err(_) => (None, None),
        };

        if hash_min.is_some() && hash.is_some() {
            app.database
                .add_meme_hashes(&meme.uuid, &hash.unwrap(), &hash_min.unwrap());

            info!("Updated hashes for = {}", &meme.uuid);
        } else {
            error!("Failed to update hashes for = {}", &meme.uuid);
        }

        sleep(Duration::from_secs(1));
    }

    Ok(())
}

pub async fn update_users(
    bot: &Bot,
    app: &Application,
    chat_id: i64,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let uids = app.database.get_users_ids_not_in_table()?;

    info!("Count updating users = {}", uids.len());

    for uid in &uids {
        info!("Sending request for user id = {uid}");
        let res = bot
            .get_chat_member(ChatId(chat_id), UserId(*uid as u64))
            .await;

        let member = match res {
            Ok(m) => m,
            Err(e) => {
                error!("User not found: {e}");

                continue;
            }
        };

        info!("Add user {uid} to database ({})", member.user.full_name());

        let _ = app.database.add_user(&AddUser::new_from_tg(&member.user));

        sleep(Duration::from_secs(1));
    }

    Ok(())
}

pub fn compare_hashes(hash1: &str, hash2: &str) -> f64 {
    let diffs_num = hash1
        .chars()
        .zip(hash2.chars())
        .filter(|(c1, c2)| c1 != c2)
        .count();

    ((hash1.len() - diffs_num) as f64 / hash1.len() as f64) * 100f64
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
