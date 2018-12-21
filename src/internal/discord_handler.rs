use eventstore::{Connection, EventData};
use futures::Future;
use serenity::command;
use serenity::{model::channel::Message, prelude::*};
use std::sync::mpsc::SyncSender;
use typemap::Key;

use super::event_consumer::EventConsumer;
use super::event_queue::{Event, WithdrawCommand};
use super::events::IncomeReceived;
use super::models::Income;

pub struct SenderWrapper;

impl Key for SenderWrapper {
    type Value = SyncSender<Event>;
}

pub struct DiscordHandler {
    eventstore: Connection,
    self_id: u64,
    et_bot_id: u64,
}

command!(withdraw(context, message, args) {
    let amount = args.single::<u64>().unwrap();
    let payload = serde_json::to_string(&WithdrawCommand {
        user_id: message.author.id.0,
        channel_id: message.channel_id.0,
        amount
    }).unwrap();
    let data = context.data.lock();
    let sender = data.get::<SenderWrapper>().unwrap();
    sender.send(Event::new("withdraw", payload)).unwrap();
});

impl DiscordHandler {
    pub fn new(
        eventstore_address: String,
        self_id: u64,
        et_bot_id: u64,
        sender: SyncSender<Event>,
    ) -> DiscordHandler {
        let eventstore = Connection::builder().start(eventstore_address).unwrap();

        let _handle = EventConsumer::new(&eventstore, sender);

        DiscordHandler {
            eventstore,
            self_id,
            et_bot_id,
        }
    }
}

impl EventHandler for DiscordHandler {
    fn message(&self, _ctx: Context, msg: Message) {
        if msg.author.id.0 == self.et_bot_id {
            if let Some(income) = Income::parse(&msg, self.self_id) {
                let event = EventData::json(
                    "income-received",
                    IncomeReceived {
                        message_id: msg.id.0,
                        channel_id: msg.channel_id.0,
                        user_id: income.user_id.0,
                        amount: income.amount,
                    },
                );

                let _ = self
                    .eventstore
                    .write_events("bank-stream")
                    .push_event(event)
                    .execute()
                    .wait()
                    .unwrap();
            }
        }
    }
}
