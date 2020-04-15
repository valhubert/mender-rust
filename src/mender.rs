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

/// Deploy an update to a device group, return the number of devices affected.
/// An error can occur if communication with the server fails,
/// if the group or the artifact is not found
/// and if command is not Deploy or token is not present.
pub fn deploy(conf: &Config) -> Result<u32, Box<dyn Error>> {
    if let (
        Command::Deploy {
            group,
            artifact,
            name,
        },
        Some(token),
    ) = (&conf.command, &conf.token)
    {
        let name = if name.is_empty() { group } else { name };
        let client = blocking_client(None)?;
        let mut page = Some(1);
        while let Some(page_idx) = page {
            let list_url = format!(
                "{}/api/management/v1/inventory/groups/{}/devices",
                conf.server_url, group
            );
            let list_devices = client
                .get(&list_url)
                .bearer_auth(token)
                .query(&[("per_page", "500"), ("page", &page_idx.to_string())])
                .send()?;
            println!("RES: {}", list_devices.text().unwrap());
            page = None;
        }
        Ok(0)
    } else {
        Err(Box::new(MenderError::new(String::from(
            "Command must be Deploy and token must be provided for deploy",
        ))))
    }
}
