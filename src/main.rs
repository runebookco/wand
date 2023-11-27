use clap::Parser;
use color_eyre::Result;
use wand::Cli;

fn main() -> Result<()> {
    let opts = Cli::parse();
    match opts.exec() {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            // TODO: Better error handling 🤡🙃
            println!("Oh dear...\n{}", error);
        }
    }

    Ok(())
}
