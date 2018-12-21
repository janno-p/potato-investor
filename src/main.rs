#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;

mod internal;

use serenity::{
    framework::{standard::help_commands, StandardFramework},
    http,
    prelude::*,
};

use std::env;
use std::sync::mpsc;

use self::internal::discord_handler::{withdraw, DiscordHandler, SenderWrapper};
use self::internal::event_queue::{start_event_queue, Event, ShowBalanceCommand};

fn main() {
    kankyo::load().expect("Failed to load .env file");

    env_logger::init();

    let discord_token = env::var("DISCORD_TOKEN").expect("Expected a token in environment");

    http::set_token(&discord_token);

    let eventstore_address =
        env::var("EVENTSTORE_ADDRESS").expect("Expected an eventstore address in environment");

    let et_bot_user_id: u64 = env::var("BOT_USER_ID")
        .expect("Expected an et-bot user id in environment")
        .parse::<u64>()
        .unwrap();

    let self_user_id: u64 = env::var("SELF_USER_ID")
        .expect("Expected a bot user id in environment")
        .parse::<u64>()
        .unwrap();

    let (sender, receiver) = mpsc::sync_channel(1024);

    let mut client = Client::new(
        &discord_token,
        DiscordHandler::new(
            eventstore_address,
            self_user_id,
            et_bot_user_id,
            sender.clone(),
        ),
    )
    .expect("Error creating client");

    {
        let mut data = client.data.lock();
        data.insert::<SenderWrapper>(sender.clone());
    }

    client.with_framework(
        StandardFramework::new()
            .configure(|c| c.prefix("!"))
            .group("potato-investor", |g| {
                g.command("balance", |c| {
                    c.desc("Kuvab kasutaja investeerimiskonto hetkeseisu")
                        .exec(|ctx, msg, _| {
                            let payload = serde_json::to_string(&ShowBalanceCommand {
                                channel_id: msg.channel_id.0,
                                user_id: msg.author.id.0,
                            })?;
                            let data = ctx.data.lock();
                            let sender = data.get::<SenderWrapper>().unwrap();
                            sender.send(Event::new("show-balance", payload))?;
                            Ok(())
                        })
                })
                .command("withdraw", |c| {
                    c.desc("Tagastab kasutajale nõutud koguses :potato:.")
                        .cmd(withdraw)
                })
            })
            .help(help_commands::with_embeds),
    );

    start_event_queue(receiver);

    if let Err(why) = client.start() {
        error!("Client error: {:?}", why);
    }
}
