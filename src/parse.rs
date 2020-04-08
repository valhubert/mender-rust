pub struct Config {
    command: Command,
    token: String,
    server_url: String,
    cert_file: String
}

#[derive(PartialEq, Debug)]
pub enum Command {
    Login { email: String },
    Deploy { group: String, artifact: String, name: String},
    GetId { serial_number: String },
    GetInfo { id: String },
    Help
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        Err("oops")
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
}
