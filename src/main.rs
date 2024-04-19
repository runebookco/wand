use std::env;

use seahorse::App;
use wand::commands::{install, login};

fn main() {
    let args: Vec<String> = env::args().collect();
    let app = App::new("Wand")
        .description("Runebook CLI")
        .author("Runebook")
        .version("0.0.1")
        .usage("wand [command]")
        .command(install::command())
        .command(login::command());

    app.run(args);
}
