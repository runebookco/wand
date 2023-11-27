use color_eyre::Result;

use crate::config::Config;

pub fn authenticate_with_runebook(config: &Config) -> Result<()> {
    let auth_uri: String = format!("http://api.runebook.local/auth/callback");
    ureq::post(&auth_uri)
        .set("content-type", "application/json")
        .send_json(ureq::json!({
            "access_token": config.access_token,
        }))?;

    Ok(())
}
