use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type Error = &'static str;

pub trait ServiceToken {
    fn current_totp(&self) -> Result<TotpToken, Error>;
}

pub trait ServicesTokens {
    fn services_tokens(&self) -> Result<HashMap<String, TotpToken>, ()>;
}

#[derive(Serialize, Deserialize)]
pub struct TotpToken {
    pub token: String,
    pub next_step_time: u64,
}
