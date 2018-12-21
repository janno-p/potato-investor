use serenity::model::{
    id::{ChannelId, UserId},
    misc::Mentionable,
};

use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::thread;

use super::events::IncomeReceived;

#[derive(Debug)]
pub struct Event {
    event_type: String,
    payload: String,
}

impl Event {
    pub fn new(event_type: &str, payload: String) -> Self {
        Event {
            event_type: String::from(event_type),
            payload,
        }
    }
}

#[derive(Debug)]
pub struct Portfolio {
    pub balance: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShowBalanceCommand {
    pub user_id: u64,
    pub channel_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WithdrawCommand {
    pub user_id: u64,
    pub channel_id: u64,
    pub amount: u64,
}

pub fn start_event_queue(receiver: Receiver<Event>) {
    thread::spawn(move || {
        let mut state: HashMap<u64, Portfolio> = HashMap::new();

        loop {
            let event = receiver.recv().unwrap();
            match event.event_type.as_str() {
                "income-received" => match serde_json::from_str::<IncomeReceived>(&event.payload) {
                    Ok(ev) => {
                        info!("Income received: {:?}", ev);
                        if let Some(x) = state.get_mut(&ev.user_id) {
                            (*x).balance = (*x).balance + ev.amount;
                        } else {
                            state.insert(ev.user_id, Portfolio { balance: ev.amount });
                        }
                        info!("Income handled: {:?}", state.get(&ev.user_id));
                    }
                    Err(_) => error!("Invalid event payload."),
                },
                "show-balance" => {
                    match serde_json::from_str::<ShowBalanceCommand>(&event.payload) {
                        Ok(ev) => {
                            let amount = match state.get(&ev.user_id) {
                                Some(x) => (*x).balance,
                                None => 0u64,
                            };
                            let user = UserId(ev.user_id);
                            let user_mention = user.mention();
                            let _ = ChannelId(ev.channel_id).send_message(|m| {
                                m.tts(true).embed(|e| {
                                    e.description(format!(
                                        "Kallis {}! Sinu investeerimiskontol on hetkel {} :potato:",
                                        user_mention, amount
                                    ))
                                })
                            });
                        }
                        Err(_) => error!("Invalid event payload."),
                    }
                }
                "withdraw" => match serde_json::from_str::<WithdrawCommand>(&event.payload) {
                    Ok(ev) => {
                        let user = UserId(ev.user_id);
                        let user_mention = user.mention();
                        let channel = ChannelId(ev.channel_id);
                        let _ = match state.get(&ev.user_id) {
                            Some(x) if (*x).balance >= ev.amount => channel.send_message(|m| {
                                m.content(format!("!give {} {}", ev.amount, user_mention))
                            }),
                            _ => channel.send_message(|m| {
                                m.tts(true).embed(|e| {
                                    e.description(format!(
                                        "Kulla {}, sul ei ole hetkel investeerimiskontol nii palju :potato:",
                                        user_mention
                                    ))
                                })
                            })
                        };
                    }
                    Err(_) => error!("Invalid event payload."),
                },
                _ => warn!("Unknown event: {:?}", event),
            }
        }
    });
}
