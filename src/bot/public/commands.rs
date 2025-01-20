use super::markups::DeleteMarkup;
use crate::app::Application;
use crate::bot::Bot;
use crate::database::entity::{
    messages::EntityTypes,
    prelude::{MemeLikes, Memes, Messages},
};
use crate::redis::RedisManager;
use std::sync::Arc;
use teloxide::types::ReplyParameters;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::{Message, Requester},
    types::InputFile,
    utils::command::BotCommands,
};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Команды которые поддерживает бот:")]
pub enum PublicCommand {
    #[command(description = "Показывает перечень команд")]
    Help,
    #[command(rename_rule = "UPPERCASE", description = "Press \"F\" to Pray Respects")]
    F,
    #[command(description = "Это даже не баян, это аккордеон на**й")]
    Accordion,
    #[command(description = "Удалить свой мем")]
    UnMeme,
    #[command(description = "Статистика мемочата")]
    Stats,
}

pub async fn help_command(bot: Bot, msg: Message, app: Arc<Application>) -> anyhow::Result<()> {
    let can_send = RedisManager::global().can_send_message("help", msg.chat.id.0, msg.id.0);
    bot.delete_message(msg.chat.id, msg.id).await?;

    if !can_send {
        return Ok(());
    }

    bot
        .send_message(
            msg.chat.id,
            format!(
                "{}\n\nЧтобы бот не посчитал твое сообщение мемом, достаточно указать в тексте сообщения к картинке <b>nomem</b>\nIssue\\Предложения: <a href=\"https://github.com/mediclab/telegram_meme_bot/issues\">писать сюда</a>\nВерсия бота: {}",
                PublicCommand::descriptions(),
                app.config.app_version
            ),
        )
        // .disable_web_page_preview(true)
        .await?;

    Ok(())
}

pub async fn f_command(bot: Bot, msg: Message) -> anyhow::Result<()> {
    let photo_id = Messages::get_random_photo(EntityTypes::PressFToPrayRespects).await;
    bot.send_photo(msg.chat.id, InputFile::file_id(&photo_id)).await?;

    Ok(())
}

pub async fn accordion_command(bot: Bot, msg: Message) -> anyhow::Result<()> {
    let me = bot.get_me().await?;
    bot.delete_message(msg.chat.id, msg.id).await?;

    match msg.reply_to_message() {
        Some(repl) => {
            if repl.from.as_ref().unwrap().id != me.id {
                return Ok(());
            }

            let can_send = RedisManager::global().can_send_message("accordion", msg.chat.id.0, msg.id.0);

            if !can_send {
                return Ok(());
            }

            let meme = match Memes::get_by_msg_id(repl.chat.id.0, repl.id.0 as u64).await {
                None => {
                    warn!("Meme not found by msg_id: {}!", repl.id.0);

                    return Ok(());
                }
                Some(m) => m,
            };
            let user_res = bot.get_chat_member(msg.chat.id, meme.user_id()).await;
            let mut user_text = String::new();

            if user_res.is_ok() {
                user_text = format!("{}!\n", crate::app::utils::get_user_text(&user_res.unwrap().user));
            }

            bot.send_message(
                msg.chat.id,
                format!("{user_text} Пользователи жалуются на великое баянище!\nЧто будем с ним делать?"),
            )
            .reply_parameters(ReplyParameters::new(repl.id))
            .reply_markup(
                DeleteMarkup::new(meme.uuid)
                    .set_ok_text("👎 Удалите, прошу прощения")
                    .set_none_text("👍 Беру на себя ответственность")
                    .get_markup(),
            )
            .await?;
        }
        None => {
            let can_send = RedisManager::global().can_send_message("accordion_none", msg.chat.id.0, msg.id.0);

            if can_send {
                bot.send_message(
                    msg.chat.id,
                    String::from("Чтобы пожаловаться на сообщение, на него нужно ответить!"),
                )
                .await?;
            }
        }
    }

    Ok(())
}

pub async fn unmeme_command(bot: Bot, msg: Message) -> anyhow::Result<()> {
    let me = bot.get_me().await?;
    bot.delete_message(msg.chat.id, msg.id).await?;

    match msg.reply_to_message() {
        Some(repl) => {
            if repl.from.as_ref().unwrap().id != me.id {
                return Ok(());
            }

            let can_send = RedisManager::global().can_send_message("unmeme", msg.chat.id.0, msg.id.0);

            if !can_send {
                return Ok(());
            }

            let meme = match Memes::get_by_msg_id(repl.chat.id.0, repl.id.0 as u64).await {
                None => {
                    warn!("Meme not found by msg_id: {}!", repl.id.0);

                    return Ok(());
                }
                Some(m) => m,
            };

            bot.send_message(msg.chat.id, String::from("Ты хочешь удалить мем?"))
                .reply_parameters(ReplyParameters::new(repl.id))
                .reply_markup(
                    DeleteMarkup::new(meme.uuid)
                        .set_ok_text("🗑 Да, я хочу удалить")
                        .set_none_text("❌ Нет, я передумал(а)")
                        .get_markup(),
                )
                .await?;
        }
        None => {
            let can_send = RedisManager::global().can_send_message("unmeme_none", msg.chat.id.0, msg.id.0);
            if can_send {
                bot.send_message(
                    msg.chat.id,
                    String::from("Чтобы удалить свой мем, нужно ответить на него!"),
                )
                .await?;
            }
        }
    }

    Ok(())
}

pub async fn stats_command(bot: Bot, msg: Message) -> anyhow::Result<()> {
    let can_send = RedisManager::global().can_send_message("stats", msg.chat.id.0, msg.id.0);
    bot.delete_message(msg.chat.id, msg.id).await?;

    if !can_send {
        return Ok(());
    }

    let memes_count = Memes::get_count(msg.chat.id.0).await;
    let mut likes_count = 0;
    let mut dislikes_count = 0;

    if let Some(like_counts) = MemeLikes::count_all(None).await {
        likes_count = like_counts.likes;
        dislikes_count = like_counts.dislikes;
    }

    let mut message = "<b>Статистика мемочата (за все время):</b>

🤡 Всего отправлено мемов: {memes_count}
❤️ Всего поставлено лайков: {memes_likes}
💔 Всего поставлено дизлайков: {memes_dislikes}"
        .to_owned();

    message = message
        .replace("{memes_count}", &memes_count.to_string())
        .replace("{memes_likes}", &likes_count.to_string())
        .replace("{memes_dislikes}", &dislikes_count.to_string());

    bot.send_message(msg.chat.id, message).await?;

    Ok(())
}
