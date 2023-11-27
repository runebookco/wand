use color_eyre::Result;
use serde::Deserialize;

use crate::{config::Config, pusher::read_from_channel};

#[derive(Deserialize, Debug)]
struct SpellExecutionResponse {
    channel: String,
}

pub fn cast_spell(config: &Config, spell: String) -> Result<()> {
    println!("Casting {}...", spell);

    let uri: String = format!("http://api.runebook.local/spells/{spell}/executions");
    let resp: SpellExecutionResponse = ureq::post(&uri)
        .set(
            "authorization",
            format!("Bearer {}", &config.access_token).as_str(),
        )
        .set("content-type", "application/json")
        .call()?
        .into_json()?;

    read_from_channel(resp.channel)?;

    Ok(())
}
