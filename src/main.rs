use std::io::stdout;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use clap::Parser;
use color_eyre::Result;
use crossterm::{cursor, ExecutableCommand};
use wand::Cli;

fn main() -> Result<()> {
    let running = Arc::new(AtomicUsize::new(0));
    let r = running.clone();

    // TODO: store ref to job id && fire off a "cancel all tasks on this job" to api on exit?

    // TODO: Can we register cleanup functions in an Arc<Vec<closures? or functions w/ stored params somehow???>>
    // And then when we get a ctrl+c signal we run those in sequence until the vec is empty and then exit(0)
    // Warp has some way of receiving references to functions so we think it should be possible 🙃
    // Another option is to have an arc that knows we have some cleanup function(s) to run so we can either
    // escape immediately or wait. This would solve the issue of needing to supply exit points all over the code.
    ctrlc::set_handler(move || {
        // Increment by 1, signaling to running processes that they should exit
        let prev = r.fetch_add(1, Ordering::SeqCst);
        if prev == 0 {
            println!("Exiting, please wait...");
        } else {
            // If we are above 0 then someone is spamming ctrl+c, so we should just kill?
            let mut stdout = stdout();
            stdout.execute(cursor::Show).unwrap();
            std::process::exit(0);
        }
    })
    .expect("Error setting Ctrl-C handler");

    let opts = Cli::parse();

    // TODO: Need to make sure ctrl+c is being checked at all loop levels, because right now
    // it will not exit if, for example, it is waiting to hear back from Pusher
    // (only once it hears back and tries to print the output will it exit)
    // ((or if you ctrl+c twice, but then you wipe your output from the screen...))

    match opts.exec(running) {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            // TODO: Better error handling 🤡🙃
            println!("Oh dear...\n{}", error);
        }
    }

    Ok(())
}
