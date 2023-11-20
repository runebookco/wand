use std::{thread::sleep, time::Duration};

use color_eyre::Result;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Auth0DeviceCodeResponse {
    pub interval: u32,
    pub device_code: String,
    pub verification_uri_complete: String,
    // user_code: String,
    // verification_uri: String,
    // expires_in: u32,
}

#[derive(Deserialize)]
pub struct Auth0AccessTokenResponse {
    pub access_token: String,
    pub id_token: String,
    // scope: String,
    // expires_in: u32,
    // token_type: String,
}

pub fn get_auth0_device_code() -> Result<Auth0DeviceCodeResponse> {
    let device_code_resp: Auth0DeviceCodeResponse =
        ureq::post("https://dev-ffhgcf1rq083t20m.us.auth0.com/oauth/device/code")
            .set("content-type", "application/x-www-form-urlencoded")
            .send_form(&[
                ("client_id", "1glLlU0sdhKP5F4pxGEfvMBaRxbPadgt"),
                ("scope", "openid profile email"),
                ("audience", "https://runebook.co/api"),
            ])?
            .into_json()?;

    Ok(device_code_resp)
}

pub fn get_auth0_access_token(
    device_code_resp: Auth0DeviceCodeResponse,
) -> Result<Auth0AccessTokenResponse> {
    println!(
        "Please visit the following link to log in: {:?}",
        device_code_resp.verification_uri_complete
    );

    // TODO: Handle errors
    // TODO: Should this have a limit to num retries?
    loop {
        match ureq::post("https://dev-ffhgcf1rq083t20m.us.auth0.com/oauth/token")
            .set("content-type", "application/x-www-form-urlencoded")
            .send_form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("device_code", &device_code_resp.device_code),
                ("client_id", "1glLlU0sdhKP5F4pxGEfvMBaRxbPadgt"),
            ]) {
            Ok(response) => {
                return Ok(Auth0AccessTokenResponse::from(response.into_json()?));
            }
            Err(ureq::Error::Status(403, _response)) => {
                sleep(Duration::from_secs(device_code_resp.interval.into()));
            }
            Err(_) => {
                println!("Transport error :(")
            }
        }
    }
}
