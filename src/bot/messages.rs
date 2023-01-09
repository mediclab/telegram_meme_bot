use teloxide::{
    prelude::*,
    types::{
        InputFile,
        ReplyMarkup,
        MessageKind,
    },
};
use std::error::Error;
use std::sync::Arc;
use rand::seq::SliceRandom;

use crate::bot::markups::*;
use crate::bot::utils as Utils;
use crate::Application;
use crate::database::repository::MemeRepository;

pub struct MessagesHandler {
    pub app: Arc<Application>,
    pub bot: Bot,
    pub msg: Message,
}

impl MessagesHandler {
    pub async fn handle(bot: Bot, msg: Message, app: Arc<Application>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let handler = MessagesHandler { app, bot, msg };

        if handler.msg.chat.id.0 > 0 {
            //return handler.private().await;
        }

        match &handler.msg.kind {
            MessageKind::Common(_) => {
                handler.common().await?;
            }
            MessageKind::NewChatMembers(_) => {
                handler.newbie().await?;
            }
            _ => {}
        }

        Ok(())
    }

    pub async fn private(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.bot.send_message(
            self.msg.chat.id,
            String::from("Временно недоступно в приватных чатах")
        ).await?;

        Ok(())
    }

    pub async fn common(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let user = self.msg.from().unwrap();
        let repository = MemeRepository::new(self.app.database.clone());
        let user_text = Utils::get_user_text(user);

        // If This is forwarded message - nothing to do.
        if self.msg.forward().is_some() {
            return Ok(());
        }

        match self.msg.photo() {
            Some(photos) => {
                // If caption contains "nomeme" - nothing to do.
                if self.msg.caption().unwrap_or("").contains("nomeme") {
                    return Ok(());
                }

                let meme = repository.add(
                    self.msg.from().unwrap().id.0 as i64,
                    self.msg.chat.id.0 as i64,
                    serde_json::json!(self.msg.photo())
                ).unwrap();

                self.bot.delete_message(self.msg.chat.id, self.msg.id).await?;

                let markup = MemeMarkup::new(0, 0, meme.uuid);
                let bot_msg = self.bot.send_photo(self.msg.chat.id, InputFile::file_id(&photos[0].file.id))
                    .caption(format!("Оцените мем {}", user_text))
                    .reply_markup(ReplyMarkup::InlineKeyboard(markup.get_markup())).await?
                    ;

                repository.add_msg_id(&meme.uuid, bot_msg.id.0 as i64);
            }
            None => {}
        }

        Ok(())
    }

    pub async fn newbie(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let newbie_msg = vec![
            "Добро пожаловать, {user_name}! С новеньких по мему, местное правило (честно, всё именно так 😊)",
            "Привет, {user_name}! Есть местное правило - с новеньких по мему. У тебя 1 час. Потом тебя удалят (честно, всё именно так 😊)",
            "Добро пожаловать, {user_name}! Ваше заявление об увольнениии принято отделом кадров, для отмены пришлите мем (честно, всё именно так 😊)",
            "Добро пожаловать, {user_name}! Подтвердите свою личность, прислав мем в этот чат.\nВсе неидентифицированные пользователи удаляются быстро - в течение 60 лет. (честно, всё именно так 😊)",
            "Добро пожаловать, {user_name}! К сожалению, ваше заявление на отпуск потеряно, следующий отпуск можно взять через 4 года 7 месяцев, для востановления заявления пришлите мем (честно, всё именно так 😊)",
            "900: {user_name}, Вас приветствует Служба безопасности Сбербанка. Для отмены операции 'В фонд озеленения Луны', Сумма: 34765.00 рублей, пришлите мем (честно, всё именно так 😊)",
            "Добро пожаловать, {user_name}! К сожалению, ваше заявление на отсрочку от мобилизации не будет принято, пока вы не пришлете мем в этот чат.",
        ];

        self.bot.delete_message(self.msg.chat.id, self.msg.id).await?;

        let users = self.msg.new_chat_members().expect("New chat members not found!");

        let a: Vec<String> = users.iter().map(|user| {
            Utils::get_user_text(user)
        }).collect();

        let message = newbie_msg.choose(&mut rand::thread_rng()).unwrap();

        self.bot.send_message(
            self.msg.chat.id,
            message.clone().replace("{user_name}", a.join(", ").as_str()),
        ).await?;

        Ok(())
    }
}