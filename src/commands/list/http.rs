use color_eyre::Result;
use ureq::serde_json::Value;

use crate::config::Config;

pub fn list_spells(config: &Config) -> Result<()> {
    println!("Listing spells...");

    let uri: String = format!("http://api.runebook.local/spells");
    let resp: Value = ureq::get(&uri)
        .set(
            "authorization",
            format!("Bearer {}", &config.access_token).as_str(),
        )
        .call()?
        .into_json()?;

    println!("{}", resp);

    Ok(())
}
