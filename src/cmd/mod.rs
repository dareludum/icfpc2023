use clap::{Parser, Subcommand};

pub mod default;
pub mod stats;

#[derive(Parser, Debug)]
#[clap()]
pub struct Args {
    #[clap(long)]
    pub batch: bool,
    #[clap(short, long, value_parser)]
    pub problems: Vec<u8>,
    #[clap(short, long)]
    pub solvers: Vec<String>,
    #[clap(subcommand)]
    pub command: Option<Commands>,
    #[clap(short, long)]
    pub log_level: Option<String>,
    #[clap(short, long)]
    pub gui: bool,
    #[clap(long)]
    pub parallel: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Stats,
    Score { problem: String, solution: String },
}
