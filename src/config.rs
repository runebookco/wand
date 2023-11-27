/* TODO: This whole thing sucks.

1. We should be using TOML
2. We need hierarchical config (user-supplied arg override > local config ?>? envar > default or whatever)
3. There are probably crates that handle some or most of this for us.
*/
use std::{
    fs::{create_dir_all, File, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
};

use color_eyre::Result;
use serde::{Deserialize, Serialize};
use ureq::serde_json;

use crate::commands::login::http::Auth0AccessTokenResponse;

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub access_token: String,
}

pub fn initialize_config(config_file_name: PathBuf) -> Result<Config> {
    // TODO: Where is this actually being stored?
    create_dir_all(".runebook")?;
    let mut buf = String::new();
    let mut config_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(config_file_name)
        .unwrap();
    config_file.read_to_string(&mut buf)?;
    if buf == "" {
        buf = String::from("{\"access_token\": \"\"}");
        println!("your config is empty just fyi");
    }

    Ok(serde_json::from_str(&buf).unwrap())
}

pub fn store_access_token_in_config(
    config: &mut Config,
    config_file_name: &PathBuf,
    access_token_resp: Auth0AccessTokenResponse,
) -> Result<()> {
    config.access_token = access_token_resp.access_token;
    let config_string = serde_json::to_string(&config).unwrap();
    let mut config_writer = File::create(&config_file_name).unwrap();
    config_writer.write_all(&config_string.as_bytes())?;

    Ok(())
}
