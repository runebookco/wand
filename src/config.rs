use std::{
    fs::{create_dir_all, OpenOptions},
    io::Read,
    path::PathBuf,
};

use color_eyre::Result;
use serde::{Deserialize, Serialize};
use ureq::serde_json;

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub access_token: String,
    pub id_token: String,
}

pub fn initialize_config(config_file_name: PathBuf) -> Result<Config> {
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
        buf = String::from("{\"access_token\": \"\", \"id_token\": \"\"}");
        println!("your config is empty just fyi");
    }

    Ok(serde_json::from_str(&buf).unwrap())
}
