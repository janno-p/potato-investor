#[derive(Debug, Serialize, Deserialize)]
pub struct IncomeReceived {
    pub message_id: u64,
    pub channel_id: u64,
    pub user_id: u64,
    pub amount: u64,
}
