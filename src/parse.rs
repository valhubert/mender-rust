use clap::{App, Arg, ArgMatches, SubCommand};

pub fn build_cli() -> App<'static, 'static> {
    App::new("mender-rust")
        .version("0.1.0")
        .author("V. Hubert <v-hubert@laposte.net>")
        .about("A small command line tool to perform tasks on a Mender server using its APIs.")
        .after_help(
            "ENVIRONMENT VARIABLES:
    SERVER_URL  Url of the mender server, must be provided
    TOKEN       Authentication token, must be provided for all subcommands except login and help
    CERT_FILE   Optional certificate for the SSL connection to the server",
        )
        .subcommand(
            SubCommand::with_name("login")
                .about("Returns a token used in other subcommands")
                .arg(
                    Arg::with_name("email")
                        .help("User email used to login to Mender server")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("getid")
                .about("Get the mender id of a device from its SerialNumber attribute")
                .arg(
                    Arg::with_name("serial number")
                        .help("SerialNumber attribute of the device")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("getinfo")
                .about("Get info of a device")
                .arg(
                    Arg::with_name("id")
                        .help("Mender id of the device")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("countartifacts")
                .about("List artifacts and count how much devices are using each"),
        )
        .subcommand(
            SubCommand::with_name("deploy")
                .about("Deploy an update to a device or to a group of devices")
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
}

impl Command {
    pub fn new(args: ArgMatches) -> Result<Command, &'static str> {
        match args.subcommand() {
            ("countartifacts", _) => Ok(Command::CountArtifacts),
            ("login", Some(sub_args)) => Ok(Command::Login {
                email: sub_args.value_of("email").unwrap().to_string(),
            }),
            ("deploy", Some(_sub_args)) => Err("deploy not handled yet!"),
            ("getid", Some(sub_args)) => Ok(Command::GetId {
                serial_number: sub_args.value_of("serial number").unwrap().to_string(),
            }),
            ("getinfo", Some(sub_args)) => Ok(Command::GetInfo {
                id: sub_args.value_of("id").unwrap().to_string(),
            }),
            _ => return Err("unrecognized or no subcommand, see help for available subcommands"),
        }
    }
}
