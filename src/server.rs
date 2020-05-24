use std::convert::Infallible;
use std::time::Duration;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};

use crate::config;
use crate::exposition::{prometheus_format, prometheus_format_down};
use crate::producer;
use crate::producer::MetricsProducer;

use anyhow::Result;

pub async fn run_server(config: &config::Config) -> Result<()> {
    let prod = MetricsProducer::new(
        &config.rpc_socket,
        Duration::from_secs(config.rate_limit),
        Duration::from_secs(config.timeout),
    )?;

    let make_service = make_service_fn(move |conn: &hyper::server::conn::AddrStream| {
        log::trace!("HTTP connection: {:?}", conn.remote_addr());
        let prod = prod.clone();
        async move { Ok::<_, Infallible>(service_fn(move |req| handle_request_log(prod.clone(), req))) }
    });

    let server = Server::bind(&config.listen).serve(make_service);

    log::info!("Listening on http://{}", config.listen);
    server.await?;
    Ok(())
}

async fn handle_request_log(
    prod: producer::MetricsProducer,
    req: Request<Body>,
) -> Result<Response<Body>> {
    let res = if req.method() == hyper::Method::GET && req.uri().path() == "/metrics" {
        handle_request_metrics(prod, req).await
    } else {
        handle_request_landing()
    };
    if let Err(e) = &res {
        log::error!("Request error: {:#}", e);
    }
    res
}

fn handle_request_landing() -> Result<Response<Body>> {
    let page = "<html>\
                    <head><title>c-lightning exporter</title></head>\
                    <body>\
                        <h1>c-lightning exporter</h1>\
                        <p><a href=\"/metrics\">Metrics</a></p>\
                    </body>\
                </html>";
    Ok(Response::builder()
        .status(200)
        .header(hyper::header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(page))?)
}

async fn handle_request_metrics(
    prod: producer::MetricsProducer,
    req: Request<Body>,
) -> Result<Response<Body>> {
    log::trace!("HTTP request: {} {}", req.method(), req.uri());

    let builder = Response::builder()
        .status(200)
        .header(hyper::header::CONTENT_TYPE, "text/plain");

    let resp = match prod.recv().await {
        Ok(r) => builder.body(Body::from(prometheus_format(r)?))?,
        Err(producer::Error::RpcError) => builder.body(Body::from(prometheus_format_down()?))?,
        Err(producer::Error::FatalError) => builder
            .status(500)
            .body(Body::from("Internal server error"))?,
    };

    log::trace!("HTTP response: {}", resp.status());
    Ok(resp)
}
