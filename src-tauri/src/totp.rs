use std::collections::HashMap;

type Error = &'static str;

pub trait ServiceToken {
    fn current_totp(&self) -> Result<String, Error>;
}

pub trait ServicesTokens {
    fn services_tokens(&self) -> Result<HashMap<String, String>, ()>;
}
