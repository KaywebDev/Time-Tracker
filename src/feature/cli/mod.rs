use error_stack::Result;
use clap::{Parser, Subcommand};
#[derive(Debug, thiserror::Error)]
#[error("A CLI error occurred")]
pub struct CliError;

#[derive(Debug, Clone, Copy, Subcommand)]
pub enum Command {
    /// Start tracking time
    Start,
    /// Stop tracking time
    Stop,
    /// Generate a report of tracked time
    Report,
}

#[derive(Parser, Debug)]
#[command(version, about, arg_required_else_help(true))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

pub fn run() -> Result<(), CliError> {
    let args: Cli = Cli::parse();

    match args.command {
        Command::Start => {
            // Implement start logic here
        }
        Command::Stop => {
            // Implement stop logic here
        }
        Command::Report => {
            // Implement report logic here
        }
    }

    Ok(())
}