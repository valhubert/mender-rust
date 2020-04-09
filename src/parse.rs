pub const HELP_STR: &str = "Help hasn't been written yet!";

pub struct Config {
    command: Command,
    token: String,
    server_url: String,
    cert_file: String,
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
    Help,
    Empty,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() == 1 {
            return Err(HELP_STR);
        }
        // Parse env token, server_url, cert_file if any
        let mut config = Config {
            command: Command::Empty,
            token: String::from(""),
            server_url: String::from(""),
            cert_file: String::from(""),
        };
        match args[1].as_str() {
            "help" => config.command = Command::Help,
            "login" => {
                if args.len() < 3 {
                    return Err("email must be provided in login command");
                }
                config.command = Command::Login {
                    email: args[2].clone(),
                };
            }
            "deploy" => {
                if args.len() < 4 {
                    return Err("group and artifact must be provided in deploy command");
                }
                config.command = Command::Deploy {
                    group: args[2].clone(),
                    artifact: args[3].clone(),
                    name: if args.len() == 5 {
                        args[4].clone()
                    } else {
                        String::new()
                    },
                };
            }
            "getid" => {
                if args.len() < 3 {
                    return Err("serial number must be provided in getid command");
                }
                config.command = Command::GetId {
                    serial_number: args[2].clone(),
                };
            }
            "getinfo" => {
                if args.len() < 3 {
                    return Err("id must be provided in getinfo command");
                }
                config.command = Command::GetInfo {
                    id: args[2].clone(),
                };
            }
            _ => return Err("unrecognized command, run help to see available commands"),
        };
        return Ok(config);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_args() {
        let args = vec![String::from("name")];
        assert!(Config::new(&args).is_err());
    }

    #[test]
    fn help_cmd() {
        let args = vec![String::from("name"), String::from("help")];
        let config = Config::new(&args).unwrap();
        assert_eq!(config.command, Command::Help);
    }

    #[test]
    fn login_no_email() {
        let args = vec![String::from("name"), String::from("login")];
        assert!(Config::new(&args).is_err());
    }

    #[test]
    fn login_email() {
        let email = String::from("toto@mail.com");
        let args = vec![String::from("name"), String::from("login"), email.clone()];
        let config = Config::new(&args).unwrap();
        assert_eq!(config.command, Command::Login { email });
    }

    #[test]
    fn deploy_no_args() {
        let args = vec![String::from("name"), String::from("deploy")];
        assert!(Config::new(&args).is_err());
    }

    #[test]
    fn deploy_group_artifact() {
        let (group, artifact) = (String::from("prod"), String::from("release"));
        let args = vec![String::from("name"), String::from("deploy"), group.clone(), artifact.clone()];
        let config = Config::new(&args).unwrap();
        assert_eq!(config.command, Command::Deploy { group, artifact, name: String::new() });
    }
}
