use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    thread::sleep,
    time::Duration,
};

use clap::{Parser, Subcommand};
use color_eyre::Result;
use serde::Deserialize;
use ureq::serde_json;

use crate::config::{initialize_config, Config};

// HTTP stuff
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

/// Wand~
#[derive(Debug, Parser)]
#[clap(name = "wand", version = "0.1")]
pub struct Cli {
    // TODO: a flattened global opts for things like output settings to control color?
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Authenticate with Runebook
    Login,
    /// Cast a spell
    Cast { spell: String },
}

impl Cli {
    // Executes commands
    //
    // Returns exit code
    pub fn exec(self) -> Result<i32> {
        match self.command {
            Command::Login => {
                let app = WandApp::new();
                app?.exec_login()?;
                Ok(0)
            }
            Command::Cast { spell } => {
                let app = WandApp::new();
                app?.exec_cast(spell)?;
                Ok(0)
            }
        }
    }
}

#[derive(Debug)]
struct WandApp {
    config: Config,
    config_file_name: PathBuf,
}

impl WandApp {
    fn new() -> Result<Self> {
        let config_file_name = Path::new(".runebook/wand_config.json");
        let config = initialize_config(config_file_name.into())?;

        Ok(Self {
            config,
            config_file_name: config_file_name.into(),
        })
    }

    fn exec_login(&mut self) -> Result<()> {
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
                    self.config.access_token = parsed_resp.access_token;
                    self.config.id_token = parsed_resp.id_token;
                    let config_string = serde_json::to_string(&self.config).unwrap();
                    let mut config_writer = File::create(&self.config_file_name).unwrap();
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

    fn exec_cast(&mut self, spell: String) -> Result<()> {
        println!("Casting {}...", spell);
        let auth_uri: String = format!("http://api.runebook.local/auth/callback");
        let auth_resp: String = ureq::post(&auth_uri)
            .set("content-type", "application/json")
            .send_json(ureq::json!({
                "id_token": &self.config.id_token,
                "access_token": &self.config.access_token,
            }))?
            .into_string()?;
        println!("{}", auth_resp);
        let uri: String = format!("http://api.runebook.local/spells/{spell}/executions");
        let resp: String = ureq::post(&uri)
            .set(
                "authorization",
                format!("Bearer {}", &self.config.access_token).as_str(),
            )
            .set("auth0-id-token", &self.config.id_token)
            .set("content-type", "application/json")
            .call()?
            .into_string()?;
        println!("{}", resp);

        Ok(())
    }
}
