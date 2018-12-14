use eventstore::{Connection, OnEventAppeared, ResolvedEvent, SubscriptionConsumer, SubscriptionEnv};
use std::thread::{JoinHandle, spawn};
use uuid::Uuid;

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