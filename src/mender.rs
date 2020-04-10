use super::parse::Config;
use std::error::Error;

/// Request an auth token from mender server
pub fn get_token(conf: &Config, login: &str) -> Result<String, Box<dyn Error>> {
    Ok(String::from("totoken"))
}
