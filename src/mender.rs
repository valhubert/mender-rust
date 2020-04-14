use super::parse::{Command, Config};
use std::error::Error;

pub const LOGIN_API: &str = "/api/management/v1/useradm/auth/login";

/// Request an auth token from mender server
pub fn get_token(conf: &Config, pass: &str) -> Result<String, Box<dyn Error>> {
    // WARNING: should add cert from Config instead of ignoring
    // all errors !!
    if let Command::Login { email } = &conf.command {
        let client = reqwest::blocking::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()?;
        let url_login = conf.server_url.clone() + LOGIN_API;
        let get_token = client
            .post(&url_login)
            .basic_auth(&email, Some(pass))
            .send()?;
        // TODO: check status code, if 200 return OK with token
        // if not parse JSON to get the error (ex: unauthorized)
        println!("{}", get_token.text().unwrap());
    } else {
        // Error
    }

    Ok(String::from("totoken"))
}
