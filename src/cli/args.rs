use clap::{ArgAction, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cap")]
#[command(about = "A tiny memo app", version)]
pub(crate) struct Cli {
    pub(crate) content: Option<String>,

    #[arg(short = 'v', long = "version", action = ArgAction::Version)]
    pub(crate) version: Option<bool>,

    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

#[derive(Subcommand)]
pub(crate) enum Command {
    Add {
        content: String,
    },
    Login {
        #[arg(long)]
        email: String,
        #[arg(long)]
        password: String,
    },
    Version,
    #[command(alias = "ls")]
    List,
}
