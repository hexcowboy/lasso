mod adapters;
mod listeners;

use std::env;

use dotenvy::dotenv;
use tokio::{
    select,
    signal::unix::{signal, SignalKind},
    sync::watch,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use url::Url;

use adapters::{Adapter, PostgresAdapter};
use listeners::EthereumListener;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // enable logging to console
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "lasso=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // create a postgres adapter
    let connection_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let adapter = PostgresAdapter::new().with_connection_url(connection_url);

    // assert that the rpc url is set and uses wss protocol
    let rpc_url =
        Url::parse(&env::var("RPC_URL").expect("RPC_URL is not set")).expect("Invalid RPC_URL");
    assert_eq!(rpc_url.scheme(), "wss", "RPC_URL must use wss protocol");

    // create an ethereum listener
    let mut listener = EthereumListener::new()
        .with_adapter(adapter)
        .with_rpc_url(rpc_url.into());

    // optionally set the starting block
    if let Some(start_height) = env::var("START_HEIGHT").ok() {
        listener =
            listener.with_starting_block(start_height.parse().expect("Invalid START_HEIGHT"));
    }

    // optionally set the reorg threshold
    if let Some(reorg_threshold) = env::var("REORG_THRESHOLD").ok() {
        listener = listener
            .with_reorg_threshold(reorg_threshold.parse().expect("Invalid REORG_THRESHOLD"));
    }

    // optionally set the time between indexes
    if let Some(time_between_indexes) = env::var("TIME_BETWEEN_INDEXES").ok() {
        listener = listener.with_time_between_indexes(
            humantime::parse_duration(&time_between_indexes).expect("Invalid TIME_BETWEEN_INDEXES"),
        );
    }

    // set the killswitch
    let (stop_tx, stop_rx) = watch::channel(());
    listener = listener.with_killswitch(stop_rx);
    listen_killswitch(stop_tx);

    // start indexing blockchain events
    listener.start().await
}

fn listen_killswitch(tx: watch::Sender<()>) {
    tokio::spawn(async move {
        let mut sigterm = signal(SignalKind::terminate()).unwrap();
        let mut sigint = signal(SignalKind::interrupt()).unwrap();
        loop {
            select! {
                _ = sigterm.recv() => tracing::warn!("Recieved SIGTERM"),
                _ = sigint.recv() => tracing::warn!("Recieved SIGINT"),
            };
            tx.send(()).unwrap();
        }
    });
}
