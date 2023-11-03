use std::{
    fs::{create_dir_all, OpenOptions},
    io::{Read, Write},
    thread::sleep,
    time::Duration,
};

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use ureq::serde_json;

#[derive(Deserialize, Serialize)]
struct Config {
    access_token: String,
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
    let mut config_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(".runebook/wand_config.toml")
        .unwrap();
    config_file.read_to_string(&mut buf)?;
    if buf == "" {
        buf = String::from("{\"access_token\": \"\"}");
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
                        ("scope", "openid profile"),
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
                        let config_string = serde_json::to_string(&config).unwrap();
                        config_file.write_all(&config_string.as_bytes())?;
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
            let uri: String = format!("http://runebook.local/api/proxy/spells/{spell}/executions");
            let resp: String = ureq::post(&uri)
                .set(
                    "authorization",
                    format!("Bearer {}", config.access_token).as_str(),
                )
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
