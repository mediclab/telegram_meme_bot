use teloxide::types::User;

pub fn get_user_text(user: &User) -> String {
    match &user.username {
        Some(uname) => format!("@{}", uname),
        None => format!("[{}](tg://user?id={})", user.first_name, user.id.0),
    }
}
