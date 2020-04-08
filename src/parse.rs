pub struct Config {
    command: Command,
    token: String,
    server_url: String,
    cert_file: String
}

pub enum Command {
    Login { email: String },
    Deploy { group: String, artifact: String, name: String},
    GetId { serial_number: String },
    GetInfo { id: String }
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        Err("oops")
    }
}
