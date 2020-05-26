use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use tokio::net::UnixStream;
use tokio::sync::broadcast;
use tokio::sync::Notify;
use tokio::time::{delay_until, timeout, Instant};
use tokio_util::compat::Tokio02AsyncReadCompatExt;

use clightningrpc::aio::LightningRPC;
use clightningrpc::responses::{GetInfo, ListFunds};

#[derive(Clone, Debug)]
pub struct LightningMetrics {
    pub getinfo: GetInfo,
    pub listfunds: ListFunds,
    pub num_nodes: u64,
    pub num_channels: u64,
}

#[derive(Clone)]
pub struct MetricsProducer {
    rx_factory: broadcast::Sender<Result<LightningMetrics, Error>>,
    notifier: Arc<Notify>,
}

/// Indicates why the producer did not send any metrics.
/// The details are intentionally not provided in order not to leak them to the http output. We
/// need to write them to log here.
#[derive(Debug, Clone)]
pub enum Error {
    /// Communication with lightning daemon failed. Tell Prometheus it is down.
    RpcError,
    /// Unexpected condition occurred. Return 500 internal server error.
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
    pub async fn recv(self) -> Result<LightningMetrics, Error> {
        let mut rx = self.rx_factory.subscribe();
        self.notifier.notify();

        for _ in 0..3 {
            match rx.recv().await {
                Ok(Ok(r)) => return Ok(r),
                Ok(Err(e)) => return Err(e),
                Err(broadcast::RecvError::Closed) => {
                    // Shouldn't happen because we keep a sender as "rx_factory" in the server.
                    log::error!("Producer closed channel!");
                    return Err(Error::FatalError);
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
        let (b_tx, _rx) = broadcast::channel(1);

        let notifier = Arc::new(Notify::new());

        tokio::spawn(producer_loop(
            notifier.clone(),
            b_tx.clone(),
            PathBuf::from(socket_path),
            min_period,
            timeout_duration,
        ));

        log::trace!("Producer: spawned");
        Ok(MetricsProducer {
            rx_factory: b_tx,
            notifier: notifier,
        })
    }
}

async fn producer_loop(
    notifier: Arc<Notify>,
    tx: broadcast::Sender<Result<LightningMetrics, Error>>,
    socket_path: PathBuf,
    min_period: Duration,
    timeout_duration: Duration,
) {
    loop {
        log::trace!("Producer: waiting for notification");
        notifier.notified().await;
        log::trace!("Producer: woken up");

        let started = Instant::now();
        let to_send = match timeout(timeout_duration, do_rpc(&socket_path)).await {
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
}

async fn do_rpc(socket_path: &Path) -> Result<LightningMetrics, anyhow::Error> {
    let stream = UnixStream::connect(socket_path)
        .await
        .context(format!("Connecting to {:?}", socket_path))?;
    let mut c = LightningRPC::new(stream.compat());
    let gi = c.getinfo().await.context("Calling getinfo")?;
    let lf = c.listfunds().await.context("Calling listfunds")?;
    // listnodes(None) and listchannels(None) are super expensive
    let ln = c.listnodes(None).await.context("Calling listnodes")?;
    let lc = c.listchannels(None).await.context("Calling listchannels")?;

    Ok(LightningMetrics {
        getinfo: gi,
        listfunds: lf,
        num_nodes: ln.nodes.len() as u64,
        num_channels: lc.channels.len() as u64,
    })
}
