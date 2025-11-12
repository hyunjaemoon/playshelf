use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Run the server in development mode
    #[arg(long, default_value_t = false)]
    pub dev: bool,
}