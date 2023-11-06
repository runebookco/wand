use std::{
    fs::{create_dir_all, File, OpenOptions},
    io::{Read, Write},
    path::Path,
    thread::sleep,
    time::Duration,
};

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use ureq::serde_json;

#[derive(Deserialize, Serialize)]
struct Config {
    access_token: String,
    id_token: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct Auth0DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u32,
    interval: u32,
    verification_uri_complete: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct Auth0AccessTokenResponse {
    access_token: String,
    id_token: String,
    scope: String,
    expires_in: u32,
    token_type: String,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    /// Turn debugging on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Authenticate
    Login,
    /// Cast a spell
    Wave { spell: String },
}

fn main() -> Result<(), ureq::Error> {
    create_dir_all(".runebook")?;
    let mut buf = String::new();
    let config_file_name = Path::new(".runebook/wand_config.json");
    let mut config_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(config_file_name)
        .unwrap();
    config_file.read_to_string(&mut buf)?;
    if buf == "" {
        buf = String::from("{\"access_token\": \"\", \"id_token\": \"\"}");
        println!("your config is empty just fyi");
    }
    let mut config: Config = serde_json::from_str(&buf).unwrap();

    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Login) => {
            let device_code_resp: Auth0DeviceCodeResponse =
                ureq::post("https://dev-ffhgcf1rq083t20m.us.auth0.com/oauth/device/code")
                    .set("content-type", "application/x-www-form-urlencoded")
                    .send_form(&[
                        ("client_id", "1glLlU0sdhKP5F4pxGEfvMBaRxbPadgt"),
                        ("scope", "openid profile email"),
                        ("audience", "https://runebook.co/api"),
                    ])?
                    .into_json()?;
            println!(
                "Please visit the following link to log in: {:?}",
                device_code_resp.verification_uri_complete
            );

            println!("Waiting...");
            for _ in 1..10 {
                match ureq::post("https://dev-ffhgcf1rq083t20m.us.auth0.com/oauth/token")
                    .set("content-type", "application/x-www-form-urlencoded")
                    .send_form(&[
                        ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                        ("device_code", &device_code_resp.device_code),
                        ("client_id", "1glLlU0sdhKP5F4pxGEfvMBaRxbPadgt"),
                    ]) {
                    Ok(response) => {
                        let parsed_resp = Auth0AccessTokenResponse::from(response.into_json()?);
                        println!("{}", parsed_resp.access_token);
                        config.access_token = parsed_resp.access_token;
                        config.id_token = parsed_resp.id_token;
                        let config_string = serde_json::to_string(&config).unwrap();
                        let mut config_writer = File::create(config_file_name).unwrap();
                        config_writer.write_all(&config_string.as_bytes())?;
                        break;
                    }
                    Err(ureq::Error::Status(403, _response)) => {
                        sleep(Duration::from_secs(device_code_resp.interval.into()));
                    }
                    Err(_) => {
                        println!("Transport error :(")
                    }
                }
            }

            Ok(())
        }
        Some(Commands::Wave { spell }) => {
            println!("Casting {}...", spell);
            let auth_uri: String = format!("http://api.runebook.local/auth/callback");
            let auth_resp: String = ureq::post(&auth_uri)
                .set("content-type", "application/json")
                .send_json(ureq::json!({
                    "id_token": config.id_token,
                    "access_token": config.access_token,
                }))?
                .into_string()?;
            println!("{}", auth_resp);
            let uri: String = format!("http://api.runebook.local/spells/{spell}/executions");
            let resp: String = ureq::post(&uri)
                .set(
                    "authorization",
                    format!("Bearer {}", config.access_token).as_str(),
                )
                .set("auth0-id-token", &config.id_token)
                .set("content-type", "application/json")
                .call()?
                .into_string()?;
            println!("{}", resp);
            Ok(())
        }
        // This case is managed by `arg_required_else_help` on the Cli struct
        _ => Ok(()),
    }
}
