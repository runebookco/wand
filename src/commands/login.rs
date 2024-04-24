use std::{thread::sleep, time::Duration};

use crate::config;

use color_eyre::eyre::Result;
use serde::Deserialize;
use seahorse::{Command, Context};

pub fn command() -> Command {
    Command::new("login")
        .description("Login to Runebook")
        .usage("wand login")
        .action(action)
}

fn action(_: &Context) {
    let resp = match get_auth0_device_code() {
        Ok(r) => r,
        Err(_) => { println!("Welp. Something went wrong. Sorry 😢"); return }
    };

    let resp = match get_auth0_access_token(resp) {
        Ok(r) => r,
        Err(_) => { println!("Welp. Something went wrong. Sorry 😢"); return }
    };

    match config::save(&resp.access_token) {
        Ok(_) => return,
        Err(e) => println!("{:?}", e),
    }
}

#[derive(Debug, Deserialize)]
pub struct Auth0DeviceCodeResponse {
    pub interval: u32,
    pub device_code: String,
    pub verification_uri_complete: String,
    // user_code: String,
    // verification_uri: String,
    // expires_in: u32,
}

#[derive(Debug, Deserialize)]
pub struct Auth0AccessTokenResponse {
    pub access_token: String,
    // pub id_token: String,
    // scope: String,
    // expires_in: u32,
    // token_type: String,
}

fn get_auth0_device_code() -> Result<Auth0DeviceCodeResponse> {
    let device_code_resp: Auth0DeviceCodeResponse =
        ureq::post("https://auth.runebook.co/oauth/device/code")
            .set("content-type", "application/x-www-form-urlencoded")
            .send_form(&[
                ("client_id", "pqqnn9OzqT7MRE3T6jhQpMoo5BdFsOt9"),
                ("scope", "openid profile email"),
                ("audience", "https://api.runebook.co"),
            ])?
            .into_json()?;

    Ok(device_code_resp)
}

fn get_auth0_access_token(
    device_code_resp: Auth0DeviceCodeResponse,
) -> Result<Auth0AccessTokenResponse> {
    let url = device_code_resp.verification_uri_complete;

    println!("Visit the following link to log in: {url}");

    loop {
        let resp = ureq::post("https://auth.runebook.co/oauth/token")
            .set("content-type", "application/x-www-form-urlencoded")
            .send_form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("device_code", &device_code_resp.device_code),
                ("client_id", "pqqnn9OzqT7MRE3T6jhQpMoo5BdFsOt9"),
            ]);

        match resp {
            Ok(response) => {
                return Ok(Auth0AccessTokenResponse::from(response.into_json()?));
            }

            Err(ureq::Error::Status(403, _response)) => {
                sleep(Duration::from_secs(device_code_resp.interval.into()));
                continue;
            }

            Err(_) => {
                println!("Transport error");
                continue;
            }
        }
    }
}
