use std::collections::HashMap;
use std::io::{stdin, stdout, Write};
use std::process::{exit, Command as ShellCommand, Stdio};

use crate::config::{self, Config};
use crate::util::{color, fmt};

use handlebars::Handlebars;
use seahorse::{Command, Context};
use ureq::serde_json::Value;
use which::which;

pub fn command() -> Command {
    Command::new("install")
        .description("Install the Runebook Kubernetes Agent")
        .alias("i")
        .usage("wand install")
        .action(action)
}

fn action(_: &Context) {
    check_prerequisites();

    let config = config::load().unwrap();
    let org = get_organization(config);
    let manifest = get_manifest();

    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("manifest", manifest).unwrap();

    let data = HashMap::from([
        ("agent_id", org["uuid"].clone()),
        ("public_key", org["public_key"].clone()),
        ("private_key", org["private_key"].clone())
    ]);

    let manifest = handlebars.render("manifest", &data).unwrap();

    diff(&manifest);
    confirm();
    //install(&manifest);
}

fn diff(manifest: &String) {
   let echo = ShellCommand::new("sh")
        .arg("-c")
        .arg(format!("echo {manifest}"))
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let kubectl = ShellCommand::new("kubectl")
        .args(["diff", "-f", "-"])
        .stdin(Stdio::from(echo.stdout.unwrap()))
        .stdout(Stdio::piped())
        .output()
        .unwrap();

    let diff = String::from_utf8(kubectl.stdout).unwrap();
    let diff = fmt::indent(color::diff(diff), 4);

    println!("\n  Continuing will create/update the following Kubernetes resources:");
    println!("  -----------------------------------------------------------------\n");
    println!("{diff}");
}

fn confirm() {
    let mut cont = String::new();

    print!("  Do you wish to continue? [Y/n] ");

    stdout().flush().unwrap();
    stdin().read_line(&mut cont).unwrap();

    match cont.as_str() {
        "y" | "Y" | "\n" | "\r" => (), 
        _ => exit(0),
    }
}

fn install(manifest: &String) {
   let echo = ShellCommand::new("sh")
        .arg("-c")
        .arg(format!("echo {manifest}"))
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    ShellCommand::new("kubectl")
        .args(["apply", "-f", "-"])
        .stdin(Stdio::from(echo.stdout.unwrap()))
        .stdout(Stdio::inherit())
        .spawn()
        .unwrap();
}

fn check_prerequisites() {
    if which("kubectl").is_err() {
        panic!("Cannot find `kubectl`. Please make sure it's available in $PATH.");
    }

    let kubectl_connected = ShellCommand::new("kubectl")
        .arg("cluster-info")
        .stdout(Stdio::piped())
        .status()
        .unwrap();

    if !kubectl_connected.success() {
        panic!("`kubectl` cannot access your cluster.");
    }
}

fn get_organization(config: Config) -> Value {
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

    match org.into_json() {
        Ok(o) => o,
        Err(_) => panic!("User error. Please try again."),
    }
}

fn get_manifest() -> String {
    let manifest = match ureq::get("http://api.runebook.local/install-manifest").call() {
        Ok(m) => m,
        Err(_) => panic!("Install manifest error. Please try again."),
    };

    match manifest.into_string() {
        Ok(m) => m,
        Err(_) => panic!("Manifest error. Please try again."),
    }
}
