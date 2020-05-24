use clap::{AppSettings, Clap};
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Debug, Clap)]
#[clap(
    about,
    version,
    setting = AppSettings::ColorNever,
    setting = AppSettings::DeriveDisplayOrder,
)]
pub struct Config {
    #[clap(
        short = "s",
        long,
        value_name = "PATH",
        about = "Path to lightning-rpc socket"
    )]
    pub rpc_socket: PathBuf,

    #[clap(
        short = "l",
        long,
        default_value = "127.0.0.1:9393",
        value_name = "ADDR:PORT",
        parse(try_from_str),
        about = "Address:port on which to expose metrics"
    )]
    pub listen: SocketAddr,

    #[clap(
        short = "r",
        long,
        default_value = "1",
        value_name = "SECONDS",
        about = "Minimal period between lightningd scrapes"
    )]
    pub rate_limit: u64,

    #[clap(
        short = "t",
        long,
        default_value = "5",
        value_name = "SECONDS",
        about = "Timeout for socket operations"
    )]
    pub timeout: u64,

    #[clap(
        short = "v",
        long,
        parse(from_occurrences),
        about = "Enable debug log messages"
    )]
    pub verbose: usize,

    #[clap(
        short = "T",
        long = "no-log-timestamps",
        parse(from_flag = std::ops::Not::not),
        about = "Do not prepend timestamps to log output",
    )]
    pub log_timestamps: bool,
}
