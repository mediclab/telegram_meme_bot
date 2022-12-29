use teloxide::{
    prelude::*, types::{
        InputFile,
        ReplyMarkup,
        MessageKind,
    }
};
use std::error::Error;
use std::sync::Arc;
use rand::seq::SliceRandom;
use crate::bot::markups::*;

use crate::BotState;
use crate::database::repository::MemeRepository;

pub async fn message_handle(bot: Bot, msg: Message, state: Arc<BotState>) -> Result<(), Box<dyn Error + Send + Sync>> {
    if msg.chat.id.0 > 0 {
        bot.send_message(msg.chat.id, String::from("Временно недоступно в приватных чатах")).await?;
        Err("Temporary disabled in private chats")?
    }

    match &msg.kind {
        MessageKind::Common(_) => {
            handle_common(&bot, &msg, &state).await?;
        },
        MessageKind::NewChatMembers(_) => {
            handle_mewbie(&bot, &msg).await?;
        },
        _ => {}
    }
    
    Ok(())
}

async fn handle_common(bot: &Bot, msg: &Message, state: &Arc<BotState>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let user = msg.from().unwrap();
    let repository = MemeRepository::new(state.db_manager.clone());

    let user_text = match &user.username {
        Some(uname) => format!("@{}", uname),
        None => format!("[{}](tg://user?id={})", user.first_name, user.id.0)
    };

    if msg.forward().is_some() {
        return Ok(());
    }

    match msg.photo() {
        Some(photos) => {
            if msg.caption().unwrap_or("").contains("nomeme") {
                Err("Message with photo contains NOMEME keyword")?
            }

            let meme = repository.add(&msg).unwrap();

            bot.delete_message(msg.chat.id, msg.id).await?;

            let markup = MemeMarkup::new(0, 0, meme.uuid);
            let bot_msg = bot.send_photo(msg.chat.id, InputFile::file_id(&photos[0].file.id))
                .caption(format!("Оцените мем {}", user_text))
                .reply_markup(ReplyMarkup::InlineKeyboard(markup.get_markup())).await?
            ;

            repository.add_msg_id(&meme.uuid, &bot_msg);
        },
        None => {}
    }

    Ok(())
}

async fn handle_mewbie(bot: &Bot, msg: &Message) -> Result<(), Box<dyn Error + Send + Sync>> {
    bot.delete_message(msg.chat.id, msg.id).await?;

    let users = msg.new_chat_members().expect("New chat members not found!");

    let _ = users.iter().map(| user | async move {
        let newbie_msg = vec![
            "Добро пожаловать, {user_name}! С новеньких по мему, местное правило (честно, всё именно так 😊)",
            "Привет, {user_name}! Есть местное правило - с новеньких по мему. У тебя 1 час. Потом тебя удалят (честно, всё именно так 😊)",
            "Добро пожаловать, {user_name}! Ваше заявление об увольнениии принято отделом кадров, для отмены пришлите мем (честно, всё именно так 😊)",
            "Добро пожаловать, {user_name}! Подтвердите свою личность, прислав мем в этот чат.\nВсе неидентифицированные пользователи удаляются быстро - в течение 60 лет. (честно, всё именно так 😊)",
            "Добро пожаловать, {user_name}! К сожалению, ваше заявление на отпуск потеряно, следующий отпуск можно взять через 4 года 7 месяцев, для востановления заявления пришлите мем (честно, всё именно так 😊)",
            "900: {user_name}, Вас приветствует Служба безопасности Сбербанка. Для отмены операции 'В фонд озеленения Луны', Сумма: 34765.00 рублей, пришлите мем (честно, всё именно так 😊)",
            "Добро пожаловать, {user_name}! К сожалению, ваше заявление на отсрочку от мобилизации не будет принято, пока вы не пришлете мем в этот чат."
        ];

        let user_text = match &user.username {
            Some(uname) => format!("@{}", uname),
            None => format!("[{}](tg://user?id={})", user.first_name, user.id.0)
        };

        let rng = &mut rand::thread_rng();
        let message = newbie_msg.choose(rng).unwrap().clone();

        let _ = bot
            .send_message(msg.chat.id, message.replace("{user_name}", user_text.as_str()))
            .await?
        ;

        Ok::<(), Box<dyn Error + Send + Sync>>(())
    });

    Ok(())
}