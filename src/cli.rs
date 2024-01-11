use std::{
    path::{Path, PathBuf},
    sync::atomic::AtomicUsize,
    sync::Arc,
};

use clap::{Parser, Subcommand};
use color_eyre::Result;

use crate::{
    commands::{
        cast::http::cast_spell,
        list::http::list_spells,
        login::http::{
            get_auth0_access_token, get_auth0_device_code, Auth0AccessTokenResponse,
            Auth0DeviceCodeResponse,
        },
    },
    config::{initialize_config, store_access_token_in_config, Config},
    http::authenticate_with_runebook,
};

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
    Cast {
        spell: String,
    },
    // List all available spells
    List,
}

impl Cli {
    // Executes commands
    //
    // Returns exit code
    pub fn exec(self, running: Arc<AtomicUsize>) -> Result<i32> {
        match self.command {
            Command::Login => {
                let mut app = WandApp::new()?;
                app.exec_login()?;
                Ok(0)
            }
            Command::Cast { spell } => {
                let mut app = WandApp::new()?;
                app.exec_cast(spell, running)?;
                Ok(0)
            }
            Command::List => {
                let mut app = WandApp::new()?;
                app.exec_list()?;
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
        // TODO: Where is this actually being stored?
        let config_file_name = Path::new(".runebook/wand_config.json");
        let config = initialize_config(config_file_name.into())?;

        Ok(Self {
            config,
            config_file_name: config_file_name.into(),
        })
    }

    fn exec_login(&mut self) -> Result<()> {
        let device_code_resp: Auth0DeviceCodeResponse = get_auth0_device_code()?;
        let access_token_resp: Auth0AccessTokenResponse = get_auth0_access_token(device_code_resp)?;
        // TODO: Make updating config less bespoke, should just be one function to key replace basically
        store_access_token_in_config(&mut self.config, &self.config_file_name, access_token_resp)?;

        Ok(())
    }

    fn exec_cast(&mut self, spell: String, running: Arc<AtomicUsize>) -> Result<()> {
        authenticate_with_runebook(&self.config)?;
        cast_spell(&self.config, spell, running)?;

        Ok(())
    }

    fn exec_list(&mut self) -> Result<()> {
        authenticate_with_runebook(&self.config)?;
        list_spells(&self.config)?;

        Ok(())
    }
}
