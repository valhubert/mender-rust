use super::parse::{Command, Config};
use std::error::Error;
use std::fmt::Display;

pub const LOGIN_API: &str = "/api/management/v1/useradm/auth/login";

#[derive(Debug)]
pub struct MenderError {
    err: String,
}

impl Display for MenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}", self.err)
    }
}

impl Error for MenderError {}

impl MenderError {
    fn new(err: String) -> MenderError {
        MenderError { err }
    }
}

fn blocking_client(
    cert: Option<reqwest::Certificate>,
) -> reqwest::Result<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
}

/// Request an auth token from mender server, it should be called
/// with a Login command otherwise an error is returned.
pub fn get_token(conf: &Config, pass: &str) -> Result<String, Box<dyn Error>> {
    // WARNING: should add cert from Config instead of ignoring
    // all errors !!
    if let Command::Login { email } = &conf.command {
        let client = blocking_client(None)?;
        let url_login = conf.server_url.clone() + LOGIN_API;
        let get_token = client
            .post(&url_login)
            .basic_auth(&email, Some(pass))
            .send()?;
        if get_token.status().is_success() {
            Ok(get_token.text().unwrap())
        } else {
            Err(Box::new(MenderError::new(format!(
                "login error. Status code '{}' response '{}'",
                get_token.status(),
                get_token.text().unwrap()
            ))))
        }
    } else {
        Err(Box::new(MenderError::new(String::from(
            "Command must be Login for get_token",
        ))))
    }
}
