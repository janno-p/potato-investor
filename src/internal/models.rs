use regex::Regex;

use serenity::model::{
    channel::{Embed, Message},
    id::UserId
};

pub struct Income {
    pub user_id: UserId,
    pub amount: u64,
}

impl Income {
    pub fn parse(message: &Message, self_user_id: u64) -> Option<Self> {
        lazy_static! {
            static ref re: Regex = Regex::new(r"^<@(\d+)> kinkis kasutajale <@(\d+)> (\d+) :potato:\.$").unwrap();
        }
        if let [Embed { description: Some(ref d), .. }] = message.embeds[..] {
            if let Some(cap) = re.captures(d.as_str()) {
                let source = (&cap[1]).parse::<u64>().unwrap();
                let target = (&cap[2]).parse::<u64>().unwrap();
                let amount = (&cap[3]).parse::<u64>().unwrap();
                if target == self_user_id {
                    return Some(Income { user_id: UserId(source), amount });
                }
            }
        }
        None
    }
}