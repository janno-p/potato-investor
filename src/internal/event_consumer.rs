use eventstore::{Connection, OnEventAppeared, ResolvedEvent, SubscriptionConsumer, SubscriptionEnv};
use std::thread::{JoinHandle, spawn};
use std::sync::mpsc::SyncSender;
use uuid::Uuid;

use super::event_queue::Event;

pub struct EventConsumer {
    sender: SyncSender<Event>,
}

impl EventConsumer {
    pub fn new(eventstore: &Connection, sender: SyncSender<Event>) -> JoinHandle<EventConsumer> {
        let subscription = eventstore.subscribe_to_stream_from("bank-stream")
            .execute();

        spawn(move || {
            subscription.consume(EventConsumer {
                sender
            })
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
                let ev = Event::new(e.event_type.to_string(), String::from_utf8(e.data.to_vec()).unwrap());
                let _ = self.sender.send(ev).unwrap();
            },
            _ => warn!("No recorded event.")
        }
        OnEventAppeared::Continue
    }

    fn when_dropped(&mut self) {

    }
}