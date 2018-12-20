use std::collections::HashMap;
use std::thread;
use std::sync::mpsc::Receiver;

use super::events::IncomeReceived;

#[derive(Debug)]
pub struct Event {
    event_type: String,
    payload: String,
}

impl Event {
    pub fn new(event_type: String, payload: String) -> Self {
        Event {
            event_type,
            payload
        }
    }
}

#[derive(Debug)]
pub struct Portfolio {
    pub balance: u64,
}

pub fn start_event_queue(receiver: Receiver<Event>) {
    thread::spawn(move || {
        let mut state: HashMap<u64, Portfolio> = HashMap::new();

        loop {
            let event = receiver.recv().unwrap();
            match event.event_type.as_str() {
                "income-received" => {
                    match serde_json::from_str::<IncomeReceived>(&event.payload) {
                        Ok(ev) => {
                            info!("Income received: {:?}", ev);
                            if let Some(x) = state.get_mut(&ev.user_id) {
                                (*x).balance = (*x).balance + ev.amount;
                            } else {
                                state.insert(ev.user_id, Portfolio { balance: ev.amount });
                            }
                            info!("Income handled: {:?}", state.get(&ev.user_id));
                        },
                        Err(_) => error!("Invalid event payload.")
                    }
                },
                _ => warn!("Unknown event: {:?}", event)
            }
        }
    });
}
