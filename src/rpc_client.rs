use futures::{SinkExt, StreamExt};

use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{from_str, json, to_string, Value};

use tokio::net::UnixStream;
use tokio_util::codec::{Framed, LinesCodec};

use anyhow::{anyhow, Result};

pub struct RpcClient {
    stream: Framed<UnixStream, LinesCodec>,
    next_id: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Response {
    jsonrpc: String,
    id: u32,
    result: Option<Value>,
    error: Option<Value>,
}

impl RpcClient {
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let sock = UnixStream::connect(path).await?;
        Ok(RpcClient {
            stream: Framed::new(sock, LinesCodec::new()),
            next_id: 0,
        })
    }

    pub async fn call(&mut self, method: &str, params: Value) -> Result<Value> {
        let expected_id = self.next_id;
        self.next_id += 1;

        let payload = json!({
            "jsonrpc": "2.0",
            "method": method,
            "id": expected_id,
            "params": params,
        });
        log::trace!("JSONRPC send: {:?}", payload);
        let payload = to_string(&payload)?;

        self.stream.send(payload).await?;

        while let Some(res) = self.stream.next().await {
            let res = res?;

            if res.chars().all(|c| c.is_whitespace()) {
                continue;
            }

            let resp: Response = from_str(&res)?;
            log::trace!("JSONRPC recv: {:?}", resp);

            if resp.jsonrpc != "2.0" {
                return Err(anyhow!("Invalid jsonrpc version"));
            }

            if resp.id != expected_id {
                return Err(anyhow!("Unexpected message id {}", resp.id));
            }

            if let Some(e) = resp.error {
                return Err(anyhow!("Responded with error: {}", e));
            }

            if let Some(r) = resp.result {
                return Ok(r);
            } else {
                return Err(anyhow!("Response has no result"));
            }
        }
        return Err(anyhow!("connection closed"));
    }

    pub async fn close(self) -> Result<()> {
        self.stream.get_ref().shutdown(std::net::Shutdown::Both)?;
        Ok(())
    }
}
