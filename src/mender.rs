use super::parse::{Command, Config};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::Display;
use std::io::Write;

pub const LOGIN_API: &str = "/api/management/v1/useradm/auth/login";
pub const DEPLOY_API: &str = "/api/management/v1/deployments/deployments";
pub const GET_DEVICES_INVENTORY_API: &str = "/api/management/v1/inventory/devices";
pub const GET_DEVICES_AUTH_API: &str = "/api/management/v2/devauth/devices";

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

#[derive(Serialize)]
struct DeployData<'a> {
    artifact_name: &'a str,
    name: &'a str,
    devices: Vec<String>,
}

/// Deploy an update to a device group, return the number of devices affected.
/// An error can occur if communication with the server fails,
/// if the group or the artifact is not found
/// and if command is not Deploy or token is not present.
pub fn deploy(conf: &Config) -> Result<usize, Box<dyn Error>> {
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
        println!(
            "Posting deployment to group {} using artifact {} and with name {}.",
            &group, &artifact, &name
        );
        let client = blocking_client(None)?;

        // List devices in the group
        let mut page = Some(1);
        let mut devices: Vec<String> = vec![];
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
            if !list_devices.status().is_success() {
                return Err(Box::new(MenderError::new(format!(
                    "deployment error, couldn't list devices in group {}. Status code '{}' response '{}'",
                    group, list_devices.status(), list_devices.text().unwrap()))));
            }
            let mut res = list_devices.json::<Vec<String>>()?;
            page = if res.len() == 0 {
                None
            } else {
                Some(page_idx + 1)
            };
            devices.append(&mut res);
        }

        // Post deployment
        let nb_devices = devices.len();
        let deploy_data = DeployData {
            artifact_name: artifact,
            name,
            devices,
        };
        let url_deploy = conf.server_url.clone() + DEPLOY_API;
        let post_deploy = client
            .post(&url_deploy)
            .bearer_auth(token)
            .json(&deploy_data)
            .send()?;
        if post_deploy.status().is_success() {
            Ok(nb_devices)
        } else {
            Err(Box::new(MenderError::new(format!(
                "deployment failed. Status code '{}' response '{}'",
                post_deploy.status(),
                post_deploy.text().unwrap()
            ))))
        }
    } else {
        Err(Box::new(MenderError::new(String::from(
            "Command must be Deploy and token must be provided for deploy",
        ))))
    }
}

#[derive(Deserialize, Debug)]
struct MenderId {
    id: String,
}

#[derive(Deserialize, Debug)]
struct MenderIdentity {
    id: String,
    identity_data: MenderSn,
}

#[derive(Deserialize, Debug)]
struct MenderSn {
    SerialNumber: String,
}

/// Get mender id of a device based on its SerialNumber attribute.
/// The command must be getid and a token must be provided.
pub fn get_id(conf: &Config) -> Result<String, Box<dyn Error>> {
    if let (Command::GetId { serial_number }, Some(token)) = (&conf.command, &conf.token) {
        println!("Searching for device with SerialNumber {}", &serial_number);

        let client = blocking_client(None)?;
        let get_device_inventory = client
            .get(&format!(
                "{}{}",
                &conf.server_url, GET_DEVICES_INVENTORY_API
            ))
            .bearer_auth(token)
            .query(&[("SerialNumber", serial_number)])
            .send()?;

        let mut res = get_device_inventory.json::<Vec<MenderId>>()?;
        if let Some(mender_id) = res.pop() {
            Ok(mender_id.id)
        } else {
            println!("SerialNumber not found in attributes, searching in identity data.");

            let mut page = Some(1);
            while let Some(page_idx) = page {
                print!(".");
                std::io::stdout().flush().unwrap();
                let get_devices_auth = client
                    .get(&format!("{}{}", &conf.server_url, GET_DEVICES_AUTH_API))
                    .bearer_auth(token)
                    .query(&[
                        ("per_page", "500"),
                        ("page", &page_idx.to_string()),
                        ("status", "accepted"),
                    ])
                    .send()?;
                let res = get_devices_auth.json::<Vec<MenderIdentity>>()?;
                let nb_results = res.len();
                if let Some(mender_identity) = res
                    .into_iter()
                    .filter(|mender_identity| {
                        mender_identity.identity_data.SerialNumber.as_str() == serial_number
                    })
                    .next()
                {
                    println!("");
                    return Ok(mender_identity.id);
                } else {
                    page = if nb_results == 0 {
                        None
                    } else {
                        Some(page_idx + 1)
                    };
                }
            }
            println!("");
            Err(Box::new(MenderError::new(String::from(
                "SerialNumber not found",
            ))))
        }
    } else {
        Err(Box::new(MenderError::new(String::from(
            "Command must be getid and token must be provided in get_id call",
        ))))
    }
}

/// Get info of a device
pub fn get_info(conf: &Config) -> Result<String, Box<dyn Error>> {
    if let (Command::GetInfo { id }, Some(token)) = (&conf.command, &conf.token) {
        let client = blocking_client(None)?;
        let get_device_inventory = client
            .get(&format!(
                "{}{}/{}",
                &conf.server_url, GET_DEVICES_INVENTORY_API, id
            ))
            .bearer_auth(token)
            .send()?;
        let json: serde_json::Value = get_device_inventory.json()?;
        Ok(serde_json::to_string_pretty(&json)?)
    } else {
        Err(Box::new(MenderError::new(String::from(
            "Command must be getinfo and token must be provided in get_info call",
        ))))
    }
}
