#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
#[macro_use] extern crate serde_json;

extern crate env_logger;
extern crate eventstore;
extern crate futures;
extern crate kankyo;
extern crate regex;
extern crate serenity;

use eventstore::{Connection, EventData};
use futures::Future;
use regex::Regex;

use serenity::{
    framework::StandardFramework,
    model::{
        channel::{Embed, Message},
        id::UserId,
    },
    prelude::*,
};

use std::env;

struct Handler;

struct Income {
    user_id: UserId,
    amount: u64,
}

impl Income {
    fn from_msg(msg: &Message) -> Option<Income> {
        lazy_static! {
            static ref re: Regex = Regex::new(r"^<@(\d+)> kinkis kasutajale <@(\d+)> (\d+) :potato:\.$").unwrap();
            static ref self_user_id: u64 = env::var("SELF_USER_ID").unwrap().parse::<u64>().unwrap();
        }
        if let [Embed { description: Some(ref d), .. }] = msg.embeds[..] {
            if let Some(cap) = re.captures(d.as_str()) {
                let source = (&cap[1]).parse::<u64>().unwrap();
                let target = (&cap[2]).parse::<u64>().unwrap();
                let amount = (&cap[3]).parse::<u64>().unwrap();
                if target == *self_user_id {
                    return Some(Income { user_id: UserId(source), amount });
                }
            }
        }
        None
    }
}

impl EventHandler for Handler {
    fn message(&self, _ctx: Context, msg: Message) {
        lazy_static! {
            static ref et_bot_user_id: u64 = env::var("BOT_USER_ID").unwrap().parse::<u64>().unwrap();
        }
        if msg.author.id.0 == *et_bot_user_id {
            if let Some(income) = Income::from_msg(&msg) {
                info!("Laekumine: {:?} ({:?})", income.user_id, income.amount);
            }
        }
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
