#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;

mod internal;
mod settings;

use serenity::{
    framework::{standard::help_commands, StandardFramework},
    http,
    prelude::*,
};

use std::process;
use std::sync::mpsc;

use crate::internal::discord_handler::{withdraw, DiscordHandler, SenderWrapper};
use crate::internal::event_queue::{start_event_queue, Event, ShowBalanceCommand};
use crate::settings::Settings;

fn main() {
    let settings = Settings::new().unwrap_or_else(|e| {
        println!("Failed to load configuration file: {}", e);
        process::exit(1);
    });

    env_logger::init();

    http::set_token(&settings.discord_token);

    let (sender, receiver) = mpsc::sync_channel(1024);

    let mut client = Client::new(
        &settings.discord_token,
        DiscordHandler::new(
            settings.eventstore_address,
            settings.self_user_id,
            settings.bot_user_id,
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
