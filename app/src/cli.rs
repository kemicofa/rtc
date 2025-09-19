use clap::{ Parser, Subcommand };

#[derive(Debug, Parser)]
#[command(name = "rtc", version, about = "Real Time Cartographer")]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Run {
        #[arg(long, env = "RTC_CONFIG")]
        config: Option<std::path::PathBuf>,
    },
    Demo,
}
