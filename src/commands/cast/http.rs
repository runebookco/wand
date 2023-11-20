use color_eyre::Result;

use crate::config::Config;

pub fn cast_spell(config: &Config, spell: String) -> Result<()> {
    println!("Casting {}...", spell);

    let uri: String = format!("http://api.runebook.local/spells/{spell}/executions");
    let resp: String = ureq::post(&uri)
        .set(
            "authorization",
            format!("Bearer {}", &config.access_token).as_str(),
        )
        .set("auth0-id-token", &config.id_token)
        .set("content-type", "application/json")
        .call()?
        .into_string()?;

    // TODO: Stream the output :)
    println!("{}", resp);

    Ok(())
}
