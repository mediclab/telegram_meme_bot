use teloxide::adaptors::DefaultParseMode;

pub mod callbacks;
pub mod commands;
pub mod markups;
pub mod messages;
pub mod top;

pub type Bot = DefaultParseMode<teloxide::Bot>;
