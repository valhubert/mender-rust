pub const HELP_STR: &str = "Help hasn't been written yet!";

pub struct Config {
    command: Command,
    token: String,
    server_url: String,
    cert_file: String,
}

impl Config {
    pub fn new(command: Command) -> Result<Config, &'static str>{
        Err("nok")
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
    Help,
    Empty,
}

impl Command {
    pub fn new(args: &[String]) -> Result<Command, &'static str> {
        if args.len() == 1 {
            return Err(HELP_STR);
        }
        match args[1].as_str() {
            "help" => Ok(Command::Help),
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
        let args = vec![String::from("name"), String::from("deploy"), group.clone(), artifact.clone()];
        let command = Command::new(&args).unwrap();
        assert_eq!(command, Command::Deploy { group, artifact, name: String::new() });
    }
}
