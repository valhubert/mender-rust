use clap::{App, Arg, SubCommand};

pub const HELP_STR: &str = "\
This is mender-rust utility, a small command line tool
to perform tasks on a Mender server using its APIs.

Available commands are:
 * help -> display this help.
 * login email -> returns a token for the given user email, password
   needs to be typed manually.
 * deploy group artifact [name] -> deploy the given artifact to the given group.
   An optional name can be given, if there are none the group name is used.
 * getid serialnumber -> get the mender id of a device based on its attribute SerialNumber.
 * getinfo id -> return info of the device with the given id.
 * countartifacts -> return a count of the artifacts used by the devices.

Used environment variables:
 * SERVER_URL -> url of the mender server, must be provided.
 * TOKEN -> authentication token, must be provided for deploy, getid, getinfo and countartifacts commands.
 * CERT_FILE -> optional verification certificate for the server secure connection.";

pub fn build_cli() -> App<'static, 'static> {
    App::new("mender-rust")
        .version("0.1.0")
        .author("V. Hubert <v-hubert@laposte.net>")
        .about("A small command line tool to perform tasks on a Mender server using its APIs.")
        .subcommand(
            SubCommand::with_name("login")
                .about("returns a token used in other subcommands")
                .arg(
                    Arg::with_name("email")
                        .help("user email used to login to Mender server")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("getid")
                .about("get the mender id of a device from its SerialNumber attribute")
                .arg(
                    Arg::with_name("serial number")
                        .help("SerialNumber attribute of the device")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("getinfo")
                .about("get info of a device")
                .arg(
                    Arg::with_name("id")
                        .help("Mender id of the device")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("countartifacts")
                .about("list artifacts and count how much devices are using each"),
        )
        .subcommand(
            SubCommand::with_name("deploy")
                .about("deploy an update to a device or to a group of devices")
                .arg(
                    Arg::with_name("group")
                        .help("Name of the group to which the update will be deployed")
                        .short("g")
                        .required_unless("device")
                        .conflicts_with("device")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("device")
                        .help("Id of the device to which the update will be deployed")
                        .short("d")
                        .required_unless("group")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("artifact")
                        .help("Name of the artifact to deploy")
                        .required(true),
                )
                .arg(
                    Arg::with_name("name")
                        .help("Name of the deployment, if not present device/group name is used"),
                ),
        )
}

pub struct Config {
    pub command: Command,
    pub token: Option<String>,
    pub server_url: String,
    pub cert_file: Option<String>,
}

impl Config {
    pub fn new(command: Command) -> Result<Config, &'static str> {
        let server_url = if let Ok(url) = std::env::var("SERVER_URL") {
            url
        } else {
            return Err("SERVER_URL env variable must be defined");
        };
        let token = if let Ok(token) = std::env::var("TOKEN") {
            Some(token)
        } else {
            None
        };
        let cert_file = if let Ok(cert) = std::env::var("CERT_FILE") {
            Some(cert)
        } else {
            None
        };
        match &command {
            Command::Deploy { .. }
            | Command::GetId { .. }
            | Command::GetInfo { .. }
            | Command::CountArtifacts
                if token == None =>
            {
                return Err(
                    "TOKEN must be provided for deploy, getid, getinfo and countartifacts commands",
                )
            }
            _ => (),
        }
        Ok(Config {
            command,
            token,
            server_url,
            cert_file,
        })
    }
}

#[derive(PartialEq, Debug)]
pub enum Command {
    Login {
        email: String,
    },
    Deploy {
        group: String,
        artifact: String,
        name: String,
    },
    GetId {
        serial_number: String,
    },
    GetInfo {
        id: String,
    },
    CountArtifacts,
    Help,
}

impl Command {
    pub fn new(args: &[String]) -> Result<Command, &'static str> {
        if args.len() == 1 {
            return Err(HELP_STR);
        }
        match args[1].as_str() {
            "help" => Ok(Command::Help),
            "countartifacts" => Ok(Command::CountArtifacts),
            "login" => {
                if args.len() < 3 {
                    return Err("email must be provided in login command");
                }
                Ok(Command::Login {
                    email: args[2].clone(),
                })
            }
            "deploy" => {
                if args.len() < 4 {
                    return Err("group and artifact must be provided in deploy command");
                }
                Ok(Command::Deploy {
                    group: args[2].clone(),
                    artifact: args[3].clone(),
                    name: if args.len() == 5 {
                        args[4].clone()
                    } else {
                        String::new()
                    },
                })
            }
            "getid" => {
                if args.len() < 3 {
                    return Err("serial number must be provided in getid command");
                }
                Ok(Command::GetId {
                    serial_number: args[2].clone(),
                })
            }
            "getinfo" => {
                if args.len() < 3 {
                    return Err("id must be provided in getinfo command");
                }
                Ok(Command::GetInfo {
                    id: args[2].clone(),
                })
            }
            _ => return Err("unrecognized command, run help to see available commands"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_args() {
        let args = vec![String::from("name")];
        assert!(Command::new(&args).is_err());
    }

    #[test]
    fn help_cmd() {
        let args = vec![String::from("name"), String::from("help")];
        let command = Command::new(&args).unwrap();
        assert_eq!(command, Command::Help);
    }

    #[test]
    fn login_no_email() {
        let args = vec![String::from("name"), String::from("login")];
        assert!(Command::new(&args).is_err());
    }

    #[test]
    fn login_email() {
        let email = String::from("toto@mail.com");
        let args = vec![String::from("name"), String::from("login"), email.clone()];
        let command = Command::new(&args).unwrap();
        assert_eq!(command, Command::Login { email });
    }

    #[test]
    fn deploy_no_args() {
        let args = vec![String::from("name"), String::from("deploy")];
        assert!(Command::new(&args).is_err());
    }

    #[test]
    fn deploy_group_artifact() {
        let (group, artifact) = (String::from("prod"), String::from("release"));
        let args = vec![
            String::from("name"),
            String::from("deploy"),
            group.clone(),
            artifact.clone(),
        ];
        let command = Command::new(&args).unwrap();
        assert_eq!(
            command,
            Command::Deploy {
                group,
                artifact,
                name: String::new()
            }
        );
    }
}
