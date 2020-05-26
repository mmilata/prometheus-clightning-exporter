extern crate log;
extern crate stderrlog;

use anyhow::Result;
use clap::Clap;
use tokio::runtime;

use prometheus_clightning_exporter::*;

fn setup_logging(config: &config::Config) -> Result<()> {
    let timestamp = if config.log_timestamps {
        stderrlog::Timestamp::Second
    } else {
        stderrlog::Timestamp::Off
    };
    stderrlog::new()
        // To enable logging from extra crates just add another call to
        // module() with the name of the crate.
        .module(module_path!())
        .color(stderrlog::ColorChoice::Never)
        .verbosity(2 + config.verbose)
        .timestamp(timestamp)
        .init()?;

    Ok(())
}

fn main() -> Result<()> {
    let c: config::Config = config::Config::parse();

    setup_logging(&c)?;

    let mut rt = runtime::Runtime::new()?;
    rt.block_on(async {
        if let Err(e) = server::run_server(&c).await {
            log::error!("Error: {:#}", e);
            panic!();
        }
    });

    Ok(())
}
