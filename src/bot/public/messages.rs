use super::markups::*;
use crate::app::Application;
use crate::bot::Bot;
use crate::database::entity::{
    messages::EntityTypes,
    prelude::{Memes, Messages, Users},
};
use crate::redis::RedisManager;
use std::sync::Arc;
use teloxide::types::{MessageKind, ReplyParameters};
use teloxide::{
    payloads::{SendMessageSetters, SendPhotoSetters},
    prelude::*,
    types::{ChatMemberKind, InputFile},
};

pub async fn chat_member_handle(bot: Bot, cm: ChatMemberUpdated) -> anyhow::Result<()> {
    let member = cm.new_chat_member;
    match member.kind {
        ChatMemberKind::Member => {
            let message = Messages::get_random_text(EntityTypes::NewbieUser).await;
            bot.send_message(
                cm.chat.id,
                message.replace("{user_name}", &crate::app::utils::get_user_text(&member.user)),
            )
            .await?;

            Users::add(member.user.into()).await;
        }
        ChatMemberKind::Left | ChatMemberKind::Banned(_) => {
            let message = Messages::get_random_text(EntityTypes::UserLeftChat).await;
            bot.send_message(
                cm.chat.id,
                message.replace("{user_name}", &crate::app::utils::get_user_text(&member.user)),
            )
            .await?;

            Users::delete(member.user.id.0 as i64).await;
        }
        _ => {}
    }

    Ok(())
}

pub async fn common(bot: Bot, msg: Message, app: Arc<Application>) -> anyhow::Result<()> {
    match msg.kind {
        MessageKind::Common(_) => {
            // If This is forwarded message - nothing to do.
            if msg.forward_origin().is_some() {
                return Ok(());
            }

            if msg.from.is_none() {
                warn!("Anonymous user detected");

                return Ok(());
            }
        }
        MessageKind::NewChatMembers(_) | MessageKind::LeftChatMember(_) => {
            bot.delete_message(msg.chat.id, msg.id).await?;

            return Ok(());
        }
        _ => return Ok(()),
    };

    // If caption contains "nomeme" - nothing to do.
    if msg.caption().unwrap_or("").to_lowercase().contains("nomem") {
        return Ok(());
    }

    if msg.photo().is_some() || msg.video().is_some() {
        if !RedisManager::global().is_chat_registered(msg.chat.id.0) {
            warn!("Chat {} is not registered", msg.chat.id.0);

            return Ok(());
        }

        if msg.photo().is_some() {
            photo_handle(&bot, &msg, &app).await?;
        }

        if msg.video().is_some() {
            video_handle(&bot, &msg).await?
        }
    }

    Ok(())
}

async fn photo_handle(bot: &Bot, msg: &Message, app: &Application) -> anyhow::Result<()> {
    let user = msg.from.as_ref().unwrap();
    let photos = if let Some(photos) = msg.photo() {
        photos
    } else {
        return Ok(());
    };
    let user_text = crate::app::utils::get_user_text(user);

    let hash_result = app.generate_hashes(&photos[0].file.id).await;
    let (Some(hash), Some(hash_min)) = hash_result.unwrap_or_else(|e| {
        warn!("Can't generate hashes. Error: {e}");

        (None, None)
    }) else {
        error!("Hashes incorrect");
        return Ok(());
    };

    let s_meme = Application::get_similar_meme(&hash_min, &hash).await;

    bot.delete_message(msg.chat.id, msg.id).await?;

    if s_meme.percent == 100 {
        let meme = s_meme.meme.unwrap();
        let message = Messages::get_random_text(EntityTypes::MemeAlreadyExists).await;

        bot.send_message(msg.chat.id, message.replace("{user_name}", &user_text))
            .reply_parameters(ReplyParameters::new(meme.msg_id()))
            .await?;

        return Ok(());
    }

    let meme = match Memes::add(msg, &Some(hash), &Some(hash_min)).await {
        None => {
            warn!("Meme is empty after insert!");
            return Ok(());
        }
        Some(m) => m,
    };

    let markup = MemeMarkup::new(0, 0, meme.uuid);
    let caption = if let Some(caption) = msg.caption() {
        format!("\n\nС подписью: {caption}")
    } else {
        String::new()
    };

    let bot_msg = bot
        .send_photo(msg.chat.id, InputFile::file_id(&photos[0].file.id))
        .caption(format!("Оцените мем {user_text}{caption}"))
        .reply_markup(markup.get_markup())
        .await?;

    meme.replace_msg_id(bot_msg.id.0 as i64).await;

    if s_meme.percent > 0 {
        let message = Messages::get_random_text(EntityTypes::SimilarMeme).await;

        bot.send_message(
            msg.chat.id,
            message.replace("{user_name}", &user_text).replace(
                "{percent}",
                &crate::app::utils::Messages::pluralize(s_meme.percent, ("процент", "процента", "процентов")),
            ),
        )
        .reply_parameters(ReplyParameters::new(s_meme.meme.unwrap().msg_id()))
        .reply_markup(
            DeleteMarkup::new(meme.uuid)
                .set_ok_text("🗑 Упс, действительно, было...")
                .set_none_text("❌ Это точно свежак!")
                .get_markup(),
        )
        .await?;
    }

    Ok(())
}

async fn video_handle(bot: &Bot, msg: &Message) -> anyhow::Result<()> {
    let user = msg.from.as_ref().unwrap();
    let video = if let Some(photos) = msg.video() {
        photos
    } else {
        return Ok(());
    };
    let user_text = crate::app::utils::get_user_text(user);

    let meme = match Memes::add(msg, &None, &None).await {
        None => {
            warn!("Meme is empty after insert!");
            return Ok(());
        }
        Some(m) => m,
    };

    bot.delete_message(msg.chat.id, msg.id).await?;

    let markup = MemeMarkup::new(0, 0, meme.uuid);
    let caption = if let Some(caption) = msg.caption() {
        format!("\n\nС подписью: {caption}")
    } else {
        String::new()
    };

    let bot_msg = bot
        .send_video(msg.chat.id, InputFile::file_id(&video.file.id))
        .caption(format!("Оцените видео-мем {user_text}{caption}"))
        .reply_markup(markup.get_markup())
        .await?;

    meme.replace_msg_id(bot_msg.id.0 as i64).await;

    Ok(())
}
