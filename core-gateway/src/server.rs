use anyhow::*;
use bytes::Bytes;
use hyper::{Method, Request, Response, StatusCode};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::upgrade;
use hyper_util::rt::TokioIo;
use http_body_util::{BodyExt, Empty, combinators::BoxBody};
use tokio::io::copy_bidirectional;
use tracing::*;
use std::convert::Infallible;
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use crate::ca::CaStore;
use crate::tls;

use crate::settings::Settings;

type RespBody = BoxBody<Bytes, hyper::Error>;

pub async fn run(cfg: Settings, _ca: CaStore) -> Result<()> {
    let addr: std::net::SocketAddr = cfg.listen.parse().context("invalid listen addr")?;
    info!(?addr, "proxy listening");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    // 并发启动 TLS 监听（先用 9443 调试）
    if let Some(tls_addr) = cfg.tls_listen.clone() {
        let ca_clone = _ca.clone();
        tokio::spawn(async move {
            if let Err(e) = run_tls(tls_addr, ca_clone).await {
                tracing::error!(err=%e, "tls listener failed");
            }
        });
    }
    loop {
        let (stream, peer) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let cfg_clone = cfg.clone();
        tokio::spawn(async move {
            let service = service_fn(move |req: Request<Incoming>| {
                let cfg = cfg_clone.clone();
                async move { handle(req, cfg).await }
            });

            if let Err(e) = hyper::server::conn::http1::Builder::new()
                .serve_connection(io, service)
                .with_upgrades()
                .await
            {
                error!(?peer, err=%e, "conn error");
            }
        });
    }
}

async fn run_tls(addr: String, ca: CaStore) -> anyhow::Result<()> {
    let acceptor = tls::build_acceptor(&ca)?;
    let listener = tokio::net::TcpListener::bind(addr.clone()).await?;
    tracing::info!(%addr, "tls listening");
    let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()?
        .https_or_http()
        .enable_http2()
        .build();
    let client: Client<_, hyper::body::Incoming> = Client::builder(TokioExecutor::new()).build(https);
    loop {
        let (stream, peer) = listener.accept().await?;
        let acceptor = acceptor.clone();
        let client = client.clone();
        tokio::spawn(async move {
            match acceptor.accept(stream).await {
                std::result::Result::Ok(tls_stream) => {
                    let io = hyper_util::rt::TokioIo::new(tls_stream);
                    if let Err(e) = hyper::server::conn::http2::Builder::new(hyper_util::rt::TokioExecutor::new())
                        .serve_connection(io, hyper::service::service_fn(move |req: Request<hyper::body::Incoming>| {
                            let client = client.clone();
                            async move {
                                // 依据 :authority/Host 选择上游
                                let host = req
                                    .headers()
                                    .get(hyper::header::HOST)
                                    .and_then(|v| v.to_str().ok())
                                    .unwrap_or("");
                                let base = if host.contains("anthropic") {
                                    "https://api.anthropic.com"
                                } else {
                                    "https://api.openai.com"
                                };
                                let path_q = req.uri().path_and_query().map(|p| p.as_str()).unwrap_or("/");
                                let uri: http::Uri = format!("{base}{path_q}").parse().unwrap();
                                let (mut parts, body) = req.into_parts();
                                parts.headers.remove(hyper::header::HOST);
                                let mut new_req = Request::from_parts(parts, body);
                                *new_req.uri_mut() = uri;
                                client.request(new_req).await
                            }
                        }))
                        .await
                    {
                        tracing::error!(?peer, err=%e, "tls h2 connection error");
                    }
                }
                std::result::Result::Err(e) => tracing::error!(?peer, err=%e, "tls accept failed"),
            }
        });
    }
}

async fn handle(req: Request<Incoming>, cfg: Settings) -> Result<Response<RespBody>> {
    if req.method() == Method::CONNECT {
        let authority = req
            .uri()
            .authority()
            .map(|a: &http::uri::Authority| a.as_str().to_string())
            .ok_or_else(|| anyhow!("CONNECT with no authority"))?;

        tokio::spawn(async move {
            match upgrade::on(req).await {
                std::result::Result::Ok(upgraded_in) => {
                    let mut client_io = TokioIo::new(upgraded_in);
                    match tokio::net::TcpStream::connect(&authority).await {
                        std::result::Result::Ok(mut server) => {
                            let _ = copy_bidirectional(&mut client_io, &mut server).await;
                        }
                        std::result::Result::Err(e) => error!(target = %authority, err=%e, "connect upstream failed"),
                    }
                }
                std::result::Result::Err(e) => error!(err=%e, "upgrade failed"),
            }
        });

        let empty = Empty::<Bytes>::new().map_err(|never| match never {});
        let resp = Response::builder()
            .status(StatusCode::OK)
            .body(empty.boxed())?;
        return Ok(resp);
    }

    // 非 CONNECT：作为显式代理入口，将 http 请求反代到上游（便于调试，无需 TLS）
    let https = HttpsConnectorBuilder::new()
        .with_native_roots()?
        .https_or_http()
        .enable_http1()
        .enable_http2()
        .build();
    let client: Client<_, hyper::body::Incoming> = Client::builder(TokioExecutor::new()).build(https);

    let host = req
        .headers()
        .get(hyper::header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let base = if host.contains("anthropic") {
        cfg.anthropic_upstream.as_deref().unwrap_or("https://api.anthropic.com")
    } else {
        cfg.openai_upstream.as_deref().unwrap_or("https://api.openai.com")
    };

    let path_q = req.uri().path_and_query().map(|p| p.as_str()).unwrap_or("/");
    let uri: http::Uri = format!("{base}{path_q}").parse().unwrap();
    let (parts, body) = req.into_parts();
    let mut new_req = Request::from_parts(parts, body);
    *new_req.uri_mut() = uri;
    let res = client.request(new_req).await?;
    let (parts, body) = res.into_parts();
    let boxed = body.boxed();
    Ok(Response::from_parts(parts, boxed))
}
