use bytes::buf::IntoBuf;
use eventstore::{Connection, OnEventAppeared, ResolvedEvent, SubscriptionConsumer, SubscriptionEnv};
use serde_json::Result;
use std::thread::{JoinHandle, spawn};
use uuid::Uuid;

use super::events::IncomeReceived;

pub struct EventConsumer;

impl EventConsumer {
    pub fn new(eventstore: &Connection) -> JoinHandle<EventConsumer> {
        let subscription = eventstore.subscribe_to_stream_from("bank-stream")
            .execute();

        spawn(move || {
            subscription.consume(EventConsumer)
        })
    }
}

impl SubscriptionConsumer for EventConsumer {
    fn when_confirmed(&mut self, _: Uuid, _: i64, _: i64) {

    }

    fn when_event_appeared<E>(&mut self, _: &mut E, e: Box<ResolvedEvent>) -> OnEventAppeared
        where E: SubscriptionEnv
    {
        match e.event {
            Some(ref e) => {
                match e.event_type.to_string().as_str() {
                    "income-received" => {
                        let result: Result<IncomeReceived> = serde_json::from_reader(e.data.clone().into_buf());
                        match result {
                            Ok(data) => info!("INCOME RECEIVED: {:?}", data),
                            Err(why) => error!("Invalid event: {:?}", why)
                        }
                    },
                    name => warn!("Unknown event type: {:?}", name)
                }
            },
            _ => warn!("No recorded event.")
        }
        OnEventAppeared::Continue
    }

    fn when_dropped(&mut self) {

    }
}