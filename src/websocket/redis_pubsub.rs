use crate::application_factory::ApplicationFactory;

use std::sync::{Arc, Mutex};

use anyhow::{anyhow as error, Result};

use tokio::sync::oneshot;

#[derive(Debug, Clone)]
pub struct PubsubPayload {
    pub channel: String,
    pub data: String,
}

pub struct RedisPubsubAdapter {
    app_fac: Arc<Mutex<ApplicationFactory>>,
    pub subscribe: String,

    keep_running_sender: Option<oneshot::Sender<String>>,
}

impl RedisPubsubAdapter {
    pub fn new(subscribe: &str, fac: Arc<Mutex<ApplicationFactory>>) -> Self {
        Self {
            app_fac: fac,
            subscribe: subscribe.to_string(),
            keep_running_sender: None,
        }
    }

    fn pubsub_loop(
        subscribe: &str,
        mut con: redis::Connection,
        sender: tokio::sync::mpsc::Sender<PubsubPayload>,
        mut keep_running_resv: oneshot::Receiver<String>,
    ) -> Result<()> {
        let mut pubsub = con.as_pubsub();
        pubsub.psubscribe(subscribe)?;

        loop {
            if let Ok(v) = keep_running_resv.try_recv() {
                if v == "stop" {
                    break;
                }
            }

            let msg = pubsub.get_message()?;

            let channel_name = msg.get_channel_name().to_string();

            let payload: std::result::Result<String, redis::RedisError> = msg.get_payload();

            if let Ok(payload) = payload {
                let p = PubsubPayload {
                    channel: channel_name.to_string(),
                    data: payload.to_string(),
                };

                if let Err(e) = tokio::runtime::Handle::current().block_on(sender.send(p)) {
                    log::error!("Pubsub sender error: {}", e.to_string());
                };

                log::info!("Got  payload: {channel_name}: {payload}");
            } else if let Err(e) = payload {
                log::error!("Got pubsub read error: {}", e.to_string());
            }
        }

        log::info!("Pubsub loop exiting for subscriber: {}...", subscribe);
        Ok(())
    }

    pub fn run(&mut self) -> Result<tokio::sync::mpsc::Receiver<PubsubPayload>> {
        let subscribe = self.subscribe.clone();
        let (tx, rx): (
            tokio::sync::mpsc::Sender<PubsubPayload>,
            tokio::sync::mpsc::Receiver<PubsubPayload>,
        ) = tokio::sync::mpsc::channel(32);

        let conn = match self.app_fac.lock() {
            Ok(app_fac) => app_fac.redis_provider.get_sync_connection()?,
            Err(e) => {
                return Err(error!(
                    "Unable to get sync connection to redis: {}",
                    e.to_string()
                ))
            }
        };

        let (keep_running_sender, keep_running_resv): (
            oneshot::Sender<String>,
            oneshot::Receiver<String>,
        ) = oneshot::channel();
        self.keep_running_sender = Some(keep_running_sender);

        tokio::task::spawn_blocking(move || {
            RedisPubsubAdapter::pubsub_loop(&subscribe, conn, tx, keep_running_resv)
        });

        Ok(rx)
    }

    pub fn stop(&mut self) -> Result<()> {
        if let Some(tx) = self.keep_running_sender.take() {
            tx.send("stop".to_string())
                .map_err(|e| error!("Could not send oneshot message: {}", e))?;
            Ok(())
        } else {
            Err(error!("Pubsub is already stopped"))
        }
    }

    pub fn is_stopped(&self) -> bool {
        self.keep_running_sender.is_none()
    }
}

impl std::ops::Drop for RedisPubsubAdapter {
    fn drop(&mut self) {
        if let Err(e) = self.stop() {
            log::info!("Redispubsub adapter: {}", e.to_string());
        }
    }
}
