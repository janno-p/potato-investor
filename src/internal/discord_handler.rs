use eventstore::{Connection, EventData};
use futures::Future;

use serenity::{
    model::channel::Message,
    prelude::*
};

use super::event_consumer::EventConsumer;
use super::models::Income;

pub struct DiscordHandler {
    eventstore: Connection,
    self_id: u64,
    et_bot_id: u64,
}

impl DiscordHandler {
    pub fn new(eventstore_address: String, self_id: u64, et_bot_id: u64) -> DiscordHandler {
        let eventstore = Connection::builder()
            .start(eventstore_address)
            .unwrap();

        let _handle = EventConsumer::new(&eventstore);

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
                let payload = json!({
                    "message_id": msg.id.0,
                    "channel_id": msg.channel_id.0,
                    "user_id": income.user_id.0,
                    "amount": income.amount
                });

                let event = EventData::json("income-received", payload);

                let _ = self.eventstore
                    .write_events("bank-stream")
                    .push_event(event)
                    .execute()
                    .wait()
                    .unwrap();
            }
        }
    }
}