use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    //noinspection ALL
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let already_exists_messages = [
            "{user_name}, кого ты хочешь нОебать? Этот мем уже был!",
            "{user_name}, ты меня не обманешь! Этот мем я уже видел!",
            "{user_name}, вздумал обмануть систему и прислать мем который уже был? Я тебя раскусил!",
            "{user_name}, я тут порылся в архивах и понял, что этот мем уже присылали!",
            "Глядите все, тут {user_name} тыренных мемов притащил!",
            "{user_name}, это же из данкест мемес канала! Мы все уже видели!",
            "Начинается постинг баянов! {user_name} объявляется главным идеологом баянов!",
            "{user_name}, ну полистай хоть канал, тут уже было такое!",
            "{user_name}, такое даже мне стыдно постить!",
            "{user_name}, я отказываюсь постить этот мем, он воняет вторичностью!",
            "{user_name}, сегодня не было, а может было?",
            "{user_name}, воровать мемы - это низко! Сделай самостоятельно!",
            "{user_name}, я вот люблю свежую пищу, свежий воздух и свежие мемы!",
        ];

        for item in already_exists_messages {
            let query = Query::insert()
                .into_table(Messages::Table)
                .columns([Messages::Type, Messages::EntityType, Messages::Message])
                .values_panic([
                    Expr::val(MessageTypes::Text.to_string()).cast_as(Alias::new("\"MessageType\"")),
                    Expr::val(EntityTypes::MemeAlreadyExists.to_string()).cast_as(Alias::new("\"MessageEntityType\"")),
                    item.into(),
                ])
                .to_owned();

            manager.exec_stmt(query).await?;
        }

        let newbie_messages = [
            "Добро пожаловать, {user_name}! С новеньких по мему, местное правило (честно, всё именно так 😊)",
            "Привет, {user_name}! Есть местное правило - с новеньких по мему. У тебя 1 час. Потом тебя удалят (честно, всё именно так 😊)",
            "Добро пожаловать, {user_name}! Ваше заявление об увольнениии принято отделом кадров, для отмены пришлите мем (честно, всё именно так 😊)",
            "Добро пожаловать, {user_name}! Подтвердите свою личность, прислав мем в этот чат",
            "Все неидентифицированные пользователи удаляются быстро - в течение 60 лет. (честно, всё именно так 😊)",
            "Добро пожаловать, {user_name}! К сожалению, ваше заявление на отпуск потеряно, следующий отпуск можно взять через 4 года 7 месяцев, для востановления заявления пришлите мем (честно, всё именно так 😊)",
            "900: {user_name}, Вас приветствует Служба безопасности Сбербанка. Для отмены операции 'В фонд озеленения Луны', Сумма: 34765.00 рублей, пришлите мем (честно, всё именно так 😊)",
            "Добро пожаловать, {user_name}! К сожалению, ваше заявление на отсрочку от мобилизации не будет принято, пока вы не пришлете мем в этот чат",
            "Если ты не голубой, нарисуй вагон другой! Добро пожаловать, {user_name}!",
            "На арене цирка новый волк! Добро пожаловать, {user_name}!",
            "Если это ваша первая ночь, в мемесовом клубе, вы обязаны постить, {user_name}!"
        ];

        for item in newbie_messages {
            let query = Query::insert()
                .into_table(Messages::Table)
                .columns([Messages::Type, Messages::EntityType, Messages::Message])
                .values_panic([
                    Expr::val(MessageTypes::Text.to_string()).cast_as(Alias::new("\"MessageType\"")),
                    Expr::val(EntityTypes::NewbieUser.to_string()).cast_as(Alias::new("\"MessageEntityType\"")),
                    item.into(),
                ])
                .to_owned();

            manager.exec_stmt(query).await?;
        }

        let similar_meme_messages = [
            "{user_name}! С точностью в {percent}, могу сказать, что этот мем уже присылали ранее!",
            "{user_name}! Меня терзают смутные сомнения, но присланный мем похож с точностью в {percent} на один из моего архива. Ты точно уверен, что это свежак?",
            "{user_name}! Мне кажется, что твой мем похож на один из моего архива. (Я уверен примерно на {percent})",
            "{user_name}! Загадка: пахнет так же, вкус такой же, кажется, это баян? (Я уверен примерно на {percent})",
            "Это же на {percent} такое же, как и вот это!",
            "Если твоей целью было запостить на {percent} похожее, то это вышло удачно, {user_name}!",
            "Мы тут не любим повторы. А это на {percent} повтор, дружище {user_name}!",
        ];

        for item in similar_meme_messages {
            let query = Query::insert()
                .into_table(Messages::Table)
                .columns([Messages::Type, Messages::EntityType, Messages::Message])
                .values_panic([
                    Expr::val(MessageTypes::Text.to_string()).cast_as(Alias::new("\"MessageType\"")),
                    Expr::val(EntityTypes::SimilarMeme.to_string()).cast_as(Alias::new("\"MessageEntityType\"")),
                    item.into(),
                ])
                .to_owned();

            manager.exec_stmt(query).await?;
        }

        let left_user_messages = [
            "💔 Ага, ага, {user_name} ливнул с нашего лампового чатика. Психика не выдержала, видимо. Либо это предатель!\nБудем скучать (нет) 🤬",
            "{user_name} исчез в страхе с нашего чатика, кто следующий?",
        ];

        for item in left_user_messages {
            let query = Query::insert()
                .into_table(Messages::Table)
                .columns([Messages::Type, Messages::EntityType, Messages::Message])
                .values_panic([
                    Expr::val(MessageTypes::Text.to_string()).cast_as(Alias::new("\"MessageType\"")),
                    Expr::val(EntityTypes::UserLeftChat.to_string()).cast_as(Alias::new("\"MessageEntityType\"")),
                    item.into(),
                ])
                .to_owned();

            manager.exec_stmt(query).await?;
        }

        manager
            .drop_table(Table::drop().table(Alias::new("__diesel_schema_migrations")).to_owned())
            .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[derive(DeriveIden)]
enum MessageTypes {
    Text,
}

#[derive(DeriveIden)]
enum EntityTypes {
    MemeAlreadyExists,
    SimilarMeme,
    NewbieUser,
    UserLeftChat,
}

#[derive(DeriveIden)]
enum Messages {
    Table,
    Type,
    EntityType,
    Message,
}
