use super::parse::{Command, Config};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::fs::File;
use std::io::{Read, Write};

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
    cert_file: &Option<String>,
) -> Result<reqwest::blocking::Client, Box<dyn Error>> {
    if let Some(cert_file) = cert_file {
        let mut buf = Vec::new();
        File::open(cert_file)?.read_to_end(&mut buf)?;
        let cert = reqwest::Certificate::from_pem(&buf)?;
        Ok(reqwest::blocking::Client::builder()
            .add_root_certificate(cert)
            .build()?)
    } else {
        Ok(reqwest::blocking::Client::builder().build()?)
    }
}

/// Request an auth token from mender server, it should be called
/// with a Login command otherwise an error is returned.
pub fn get_token(conf: &Config, pass: &str) -> Result<String, Box<dyn Error>> {
    if let Command::Login { email } = &conf.command {
        let client = blocking_client(&conf.cert_file)?;
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
        let client = blocking_client(&conf.cert_file)?;

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
#[allow(non_snake_case)]
struct MenderSn {
    SerialNumber: String,
}

/// Get mender id of a device based on its SerialNumber attribute.
/// The command must be getid and a token must be provided.
pub fn get_id(conf: &Config) -> Result<String, Box<dyn Error>> {
    if let (Command::GetId { serial_number }, Some(token)) = (&conf.command, &conf.token) {
        println!("Searching for device with SerialNumber {}", &serial_number);

        let client = blocking_client(&conf.cert_file)?;
        let get_device_inventory = client
            .get(&format!(
                "{}{}",
                &conf.server_url, GET_DEVICES_INVENTORY_API
            ))
            .bearer_auth(token)
            .query(&[("SerialNumber", serial_number)])
            .send()?;

        if !get_device_inventory.status().is_success() {
            return Err(Box::new(MenderError::new(format!(
                "searching device failed. Status code '{}' response '{}'",
                get_device_inventory.status(),
                get_device_inventory.text().unwrap()
            ))));
        }

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

                if !get_devices_auth.status().is_success() {
                    return Err(Box::new(MenderError::new(format!(
                        "searching device failed. Status code '{}' response '{}'",
                        get_devices_auth.status(),
                        get_devices_auth.text().unwrap()
                    ))));
                }

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
        let client = blocking_client(&conf.cert_file)?;
        let get_device_inventory = client
            .get(&format!(
                "{}{}/{}",
                &conf.server_url, GET_DEVICES_INVENTORY_API, id
            ))
            .bearer_auth(token)
            .send()?;

        if !get_device_inventory.status().is_success() {
            return Err(Box::new(MenderError::new(format!(
                "get info failed. Status code '{}' response '{}'",
                get_device_inventory.status(),
                get_device_inventory.text().unwrap()
            ))));
        }

        let json: serde_json::Value = get_device_inventory.json()?;
        Ok(serde_json::to_string_pretty(&json)?)
    } else {
        Err(Box::new(MenderError::new(String::from(
            "Command must be getinfo and token must be provided in get_info call",
        ))))
    }
}

#[derive(Deserialize, Debug)]
struct MenderAttribute {
    name: String,
    value: String,
}

#[derive(Deserialize, Debug)]
struct MenderDevice {
    id: String,
    attributes: Option<Vec<MenderAttribute>>,
}

impl MenderDevice {
    fn artifact_name(&self) -> String {
        if let Some(attributes) = &self.attributes {
            for attribute in attributes {
                if attribute.name == "artifact_name" {
                    return attribute.value.clone();
                }
            }
        }
        return String::new();
    }
}

/// Return the list of artifacts with a count of how much devices are using it.
pub fn count_artifacts(conf: &Config) -> Result<String, Box<dyn Error>> {
    if let (Command::CountArtifacts, Some(token)) = (&conf.command, &conf.token) {
        print!("Inventoring artifact used by devices");
        let client = blocking_client(&conf.cert_file)?;
        let mut artifacts_count = HashMap::new();
        let mut page = Some(1);
        while let Some(page_idx) = page {
            print!(".");
            std::io::stdout().flush().unwrap();
            let get_devices_inv = client
                .get(&format!(
                    "{}{}",
                    &conf.server_url, GET_DEVICES_INVENTORY_API
                ))
                .bearer_auth(token)
                .query(&[("per_page", "500"), ("page", &page_idx.to_string())])
                .send()?;

            if !get_devices_inv.status().is_success() {
                return Err(Box::new(MenderError::new(format!(
                    "searching device failed. Status code '{}' response '{}'",
                    get_devices_inv.status(),
                    get_devices_inv.text().unwrap()
                ))));
            }

            let res = get_devices_inv.json::<Vec<MenderDevice>>()?;
            let nb_devices = res.len();
            let artifacts: Vec<String> = res
                .into_iter()
                .map(|device| device.artifact_name())
                .collect();
            for artifact in artifacts {
                let count = artifacts_count.entry(artifact).or_insert(0);
                *count += 1;
            }
            page = if nb_devices > 0 {
                Some(page_idx + 1)
            } else {
                None
            };
        }
        println!("");
        Ok(display_ordered(artifacts_count))
    } else {
        Err(Box::new(MenderError::new(String::from(
            "Command must be countartifacts and token must be provided in count_artifacts call",
        ))))
    }
}

fn display_ordered(map: HashMap<String, i32>) -> String {
    let mut vec: Vec<(&String, &i32)> = map.iter().collect();
    vec.sort_by(|a, b| b.1.cmp(a.1));
    let mut disp = String::new();
    for (key, value) in vec {
        disp.push_str(&format!("{}: {}\n", key, value));
    }
    disp
}
