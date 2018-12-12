#[macro_use] extern crate log;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate serenity;

extern crate env_logger;
extern crate eventstore;
extern crate futures;
extern crate kankyo;

use eventstore::{Connection, EventData};
use futures::Future;

use serenity::{
    framework::StandardFramework,
    model::channel::Message,
    prelude::*,
};

use std::env;

struct Handler;

impl EventHandler for Handler {
    fn message(&self, _ctx: Context, msg: Message) {
        info!("{:?}", msg);
    }
}

fn main() {
    kankyo::load()
        .expect("Failed to load .env file");

    env_logger::init();

    let discord_token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in environment");

    let eventstore_address = env::var("EVENTSTORE_ADDRESS")
        .expect("Expected an eventstore address in environment");

    let connection = Connection::builder()
        .start(eventstore_address)
        .unwrap();

    let payload = json!({
        "is_rust_a_nice_language": true
    });

    let event = EventData::json("language-poll", payload);

    let _ = connection.write_events("language-stream")
        .push_event(event)
        .execute()
        .wait()
        .unwrap();

    let mut client = Client::new(&discord_token, Handler)
        .expect("Error creating client");

    client.with_framework(StandardFramework::new()
        .configure(|c| c
            .prefix("!"))
    );

    if let Err(why) = client.start() {
        error!("Client error: {:?}", why);
    }
}
