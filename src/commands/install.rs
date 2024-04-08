use std::collections::HashMap;

use crate::config;

use handlebars::Handlebars;
use seahorse::{Command, Context};
use ureq::serde_json::Value;

pub fn command() -> Command {
    Command::new("install")
        .description("Install the Runebook Kubernetes Agent")
        .alias("i")
        .usage("wand install")
        .action(action)
}

fn action(_: &Context) {
    let config = config::load().unwrap();

    if config.access_token.is_empty() {
        panic!("Please `wand login` first.");
    }

    let access_token = config.access_token.as_str();
    let bearer = format!("Bearer {}", access_token);
    
    let org = match ureq::get("http://api.runebook.local/organization")
        .set("Accept", "application/json")
        .set("Authorization", bearer.as_str())
        .call()
        {
            Ok(o) => o,
            Err(_) => panic!("Invalid User error. Please try again.")
        };

    let org: Value = match org.into_json() {
        Ok(o) => o,
        Err(_) => panic!("User error. Please try again."),
    };

    let manifest = match ureq::get("http://api.runebook.local/install-manifest").call() {
        Ok(m) => m,
        Err(_) => panic!("Install manifest error. Please try again."),
    };

    let manifest = match manifest.into_string() {
        Ok(m) => m,
        Err(_) => panic!("Manifest error. Please try again."),
    };

    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("manifest", manifest).unwrap();

    let mut data = HashMap::new();
    data.insert("agent_id", org["uuid"].clone());
    data.insert("private_key", org["private_key"].clone());

    let manifest = handlebars.render("manifest", &data).unwrap();

    println!("{manifest}");
}
