use std::{
    collections::HashSet,
    future::Future,
    net::{IpAddr, SocketAddr},
    pin::Pin,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::http::{self, Request, Response};
use dashmap::DashMap;
use tower::{Layer, Service};

pub type BanList = Arc<DashMap<IpAddr, Instant>>;

#[derive(Debug, Clone)]
pub struct BouncerConfig {
    pub blocked_paths: HashSet<String>,
    pub ban_duration: Duration,
    pub banned_status: http::StatusCode,
    pub blocked_status: http::StatusCode,
    pub log_level: tracing::Level,
}

impl BouncerConfig {
    pub fn from_rules(presets: &[&str], custom: &[&str]) -> Self {
        let blocked_paths = crate::bouncer::rules::from_rules(presets, custom);
        Self {
            blocked_paths,
            ban_duration: Duration::from_secs(3600),
            banned_status: http::StatusCode::FORBIDDEN,
            blocked_status: http::StatusCode::FORBIDDEN,
            log_level: tracing::Level::DEBUG,
        }
    }

    pub fn from_preset_rules(presets: &[&str]) -> Self {
        Self::from_rules(presets, &[])
    }

    pub fn from_custom_rules(custom: &[&str]) -> Self {
        Self::from_rules(&[], custom)
    }

    pub fn banned_response(mut self, status: http::StatusCode) -> Self {
        self.banned_status = status;
        self
    }

    pub fn blocked_response(mut self, status: http::StatusCode) -> Self {
        self.blocked_status = status;
        self
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.ban_duration = duration;
        self
    }

    pub fn log_level(mut self, level: tracing::Level) -> Self {
        self.log_level = level;
        self
    }
}

// BouncerLayer factory
#[derive(Debug, Clone)]
pub struct BouncerLayer {
    config: BouncerConfig,
    banlist: BanList,
}

impl BouncerLayer {
    pub fn new(config: BouncerConfig) -> Self {
        Self {
            config,
            banlist: Arc::new(DashMap::new()),
        }
    }

    /// Expose banlist for observability
    pub fn banlist(&self) -> Arc<DashMap<IpAddr, Instant>> {
        self.banlist.clone()
    }
}

impl<S> Layer<S> for BouncerLayer {
    type Service = BouncerMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        BouncerMiddleware {
            inner,
            config: self.config.clone(),
            banlist: self.banlist.clone(),
        }
    }
}

// A middleware that blocks paths and bans IPs
#[derive(Debug, Clone)]
pub struct BouncerMiddleware<S> {
    inner: S,
    config: BouncerConfig,
    banlist: BanList,
}

impl<ReqBody, ResBody, S> Service<Request<ReqBody>> for BouncerMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
    ResBody: Default + Send + 'static,
{
    type Response = Response<ResBody>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let config = self.config.clone();
        let banlist = self.banlist.clone();

        let ip = req
            .headers()
            .get("x-real-ip")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse().ok())
            .or_else(|| req.extensions().get::<IpAddr>().cloned())
            .or_else(|| {
                req.extensions()
                    .get::<axum::extract::ConnectInfo<SocketAddr>>()
                    .map(|info| info.0.ip())
            });

        let path = req.uri().path().to_owned();

        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            if let Some(ip) = ip {
                if let Some(&expiry) = banlist.get(&ip).as_deref() {
                    if Instant::now() < expiry {
                        log_event(
                            config.log_level,
                            &ip,
                            &path,
                            "Banned IP attempted access",
                            true,
                            false,
                        );
                        let mut res = Response::default();
                        *res.status_mut() = config.banned_status;
                        return Ok(res);
                    } else {
                        banlist.remove(&ip);
                    }
                }

                if config.blocked_paths.contains(&path) {
                    banlist.insert(ip, Instant::now() + config.ban_duration);
                    log_event(
                        config.log_level,
                        &ip,
                        &path,
                        "Blocked path accessed, IP banned",
                        false,
                        true,
                    );
                    let mut res = Response::default();
                    *res.status_mut() = config.blocked_status;
                    return Ok(res);
                }
            }

            inner.call(req).await
        })
    }
}

fn log_event(
    level: tracing::Level,
    ip: &IpAddr,
    path: &str,
    msg: &str,
    banned: bool,
    blocked: bool,
) {
    match level {
        tracing::Level::ERROR => {
            tracing::error!(ip = %ip, path = %path, banned, blocked, "{msg}")
        }
        tracing::Level::WARN => {
            tracing::warn!(ip = %ip, path = %path, banned, blocked, "{msg}")
        }
        tracing::Level::INFO => {
            tracing::info!(ip = %ip, path = %path, banned, blocked, "{msg}")
        }
        tracing::Level::DEBUG => {
            tracing::debug!(ip = %ip, path = %path, banned, blocked, "{msg}")
        }
        tracing::Level::TRACE => {
            tracing::trace!(ip = %ip, path = %path, banned, blocked, "{msg}")
        }
    }
}
