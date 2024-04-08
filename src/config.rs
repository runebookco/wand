/* TODO: This whole thing sucks.

1. We should be using TOML
2. We need hierarchical config (user-supplied arg override > local config ?>? envar > default or whatever)
3. There are probably crates that handle some or most of this for us.
*/
use std::error::Error;
use std::fs::{create_dir_all, File};
use std::io::{BufReader, Write};
use std::path::PathBuf;

use color_eyre::{eyre::eyre, Result};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use ureq::serde_json;

lazy_static! {
    static ref CONFIG_DIR: PathBuf = home_dir().unwrap().join(".wand");
    static ref CONFIG_FILE: PathBuf = CONFIG_DIR.join("config.json");
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub access_token: String,
}

pub fn save(access_token: &String) -> Result<Config> {
    match create_dir_all(CONFIG_DIR.to_owned()) {
        Ok(_) => {}
        Err(_) => return Err(eyre!("Runebook config file error")),
    }

    let mut file = match File::create(CONFIG_FILE.to_owned()) {
        Ok(f) => f,
        Err(_) => return Err(eyre!("Runebook config file error")),
    };

    let config = Config { access_token: access_token.clone() };

    let json = match serde_json::to_string(&config) {
        Ok(c) => c,
        Err(_) => return Err(eyre!("Runebook config file error")),
    };

    match file.write_all(&json.as_bytes()) {
        Ok(_) => Ok(config),
        Err(_) => Err(eyre!("Runebook config file error")),
    }
}

pub fn load() -> Result<Config, Box<dyn Error>> {
    let file = File::open(CONFIG_FILE.to_owned())?;
    let reader = BufReader::new(file);
    let config: Config = serde_json::from_reader(reader)?;
    Ok(config)
}
