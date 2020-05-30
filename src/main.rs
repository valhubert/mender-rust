use clap::{App, Arg, SubCommand};
use std::env;
use std::error::Error;
use std::process;

mod mender;
mod parse;

fn main() {
    let matches = App::new("mender-rust")
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
        .get_matches();

    let args: Vec<String> = env::args().collect();

    let command = parse::Command::new(&args).unwrap_or_else(|err| {
        println!("Parse error: {}", err);
        process::exit(1);
    });
    if command == parse::Command::Help {
        println!("{}", parse::HELP_STR);
        process::exit(0);
    }
    let config = parse::Config::new(command).unwrap_or_else(|err| {
        println!("Config error: {}", err);
        process::exit(1);
    });

    if let Err(e) = run(config) {
        println!("Run error: {}", e);
        process::exit(2);
    }
}

fn run(config: parse::Config) -> Result<(), Box<dyn Error>> {
    match config.command {
        parse::Command::Login { .. } => {
            println!("Type password:");
            let mut password = String::new();
            std::io::stdin().read_line(&mut password)?;
            println!("Token {}", mender::get_token(&config, &password.trim())?);
        }
        parse::Command::Deploy { .. } => {
            println!("Deployed to {} devices", mender::deploy(&config)?)
        }
        parse::Command::GetId { .. } => println!("Mender id is: {}", mender::get_id(&config)?),
        parse::Command::GetInfo { .. } => println!("{}", mender::get_info(&config)?),
        parse::Command::CountArtifacts => println!("{}", mender::count_artifacts(&config)?),
        _ => println!("Another command"),
    };
    Ok(())
}
