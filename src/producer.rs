use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::broadcast;
use tokio::sync::Notify;
use tokio::time::{delay_until, timeout, Instant};

use crate::rpc_client::RpcClient;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetInfoResult {
    pub id: Option<String>,
    pub alias: Option<String>,
    pub network: Option<String>,
    pub version: Option<String>,
    pub blockheight: Option<u32>,
    pub num_peers: Option<u32>,
    pub num_pending_channels: Option<u32>,
    pub num_active_channels: Option<u32>,
    pub num_inactive_channels: Option<u32>,
}

#[derive(Clone)]
pub struct MetricsProducer {
    rx_factory: broadcast::Sender<Result<GetInfoResult, Error>>,
    notifier: Arc<Notify>,
}

#[derive(Debug, Clone)]
pub enum Error {
    RpcError,
    FatalError,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::RpcError => write!(f, "RPC Error"),
            Error::FatalError => write!(f, "Fatal Error"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl MetricsProducer {
    pub async fn recv(self) -> Result<GetInfoResult, Error> {
        let mut rx = self.rx_factory.subscribe();
        self.notifier.notify();

        for _ in 0..3 {
            match rx.recv().await {
                Ok(Ok(r)) => return Ok(r),
                Ok(Err(e)) => return Err(e),
                Err(broadcast::RecvError::Closed) => {
                    // Shouldn't happen because we keep a sender as "rx_factory" in the server.
                    log::error!("Producer closed channel!");
                    return Err(Error::FatalError)
                }
                Err(broadcast::RecvError::Lagged(_)) => {
                    log::error!("Lagged channel!");
                    continue;
                }
            };
        }
        log::error!("Channel stays lagged");
        Err(Error::FatalError)
    }

    pub fn new(
        socket_path: &Path,
        min_period: Duration,
        timeout_duration: Duration,
    ) -> anyhow::Result<MetricsProducer> {
        log::trace!("Producer: spawning");
        let (tx, _rx) = broadcast::channel(1);
        let rx_factory = tx.clone();

        let n_tx = Arc::new(Notify::new());
        let n_rx = n_tx.clone();
        let pb = PathBuf::from(socket_path);

        tokio::spawn(async move {
            loop {
                log::trace!("Producer: waiting for notification");
                n_rx.notified().await;
                log::trace!("Producer: woken up");

                let started = Instant::now();
                let to_send = match timeout(timeout_duration, do_rpc(&pb)).await {
                    Ok(Ok(r)) => Ok(r),
                    Ok(Err(e)) => {
                        log::error!("RPC error: {:#}", e);
                        Err(Error::RpcError)
                    }
                    Err(_) => {
                        log::error!("RPC timed out: {:?}", timeout_duration);
                        Err(Error::RpcError)
                    }
                };
                log::trace!("Producer: sending result");

                if let Err(_) = tx.send(to_send) {
                    log::error!("Producer: no receivers");
                }
                log::trace!("Producer: sleeping");
                delay_until(started + min_period).await;
            }
        });

        log::trace!("Producer: spawned");
        Ok(MetricsProducer {
            rx_factory: rx_factory,
            notifier: n_tx,
        })
    }
}

async fn do_rpc(socket_path: &Path) -> Result<GetInfoResult, anyhow::Error> {
    let mut c = RpcClient::new(socket_path)
        .await
        .context(format!("Connecting to {:?}", socket_path))?;
    let res = c
        .call("getinfo", json!([]))
        .await
        .context("Calling getinfo")?;
    let parsed: GetInfoResult = serde_json::from_value(res).context("Parsing response JSON")?;

    Ok(parsed)
}
