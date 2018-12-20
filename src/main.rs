#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;

extern crate bytes;
extern crate env_logger;
extern crate eventstore;
extern crate futures;
extern crate kankyo;
extern crate regex;
extern crate serde_json;
extern crate serenity;

mod internal;

use serenity::{framework::StandardFramework, prelude::*, http};
use std::env;
use std::sync::mpsc;

use self::internal::discord_handler::DiscordHandler;
use self::internal::event_queue::start_event_queue;

fn main() {
    kankyo::load()
        .expect("Failed to load .env file");

    env_logger::init();

    let discord_token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in environment");

    http::set_token(&discord_token);

    let eventstore_address = env::var("EVENTSTORE_ADDRESS")
        .expect("Expected an eventstore address in environment");

    let et_bot_user_id: u64 = env::var("BOT_USER_ID")
        .expect("Expected an et-bot user id in environment")
        .parse::<u64>()
        .unwrap();

    let self_user_id: u64 = env::var("SELF_USER_ID")
        .expect("Expected a bot user id in environment")
        .parse::<u64>()
        .unwrap();

    let (sender, receiver) = mpsc::sync_channel(1024);

    let mut client = Client::new(&discord_token, DiscordHandler::new(eventstore_address, self_user_id, et_bot_user_id, sender.clone()))
        .expect("Error creating client");

    client.with_framework(StandardFramework::new()
        .configure(|c| c
            .prefix("!"))
    );

    start_event_queue(receiver);

    if let Err(why) = client.start() {
        error!("Client error: {:?}", why);
    }
}
