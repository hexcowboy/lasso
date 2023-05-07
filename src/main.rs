mod adapters;
mod listeners;

use std::env;

use dotenvy::dotenv;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

    // create an ethereum listener
    let mut listener = EthereumListener::new()
        .with_adapter(adapter)
        .with_rpc_url(env::var("RPC_URL").expect("RPC_URL is not set"));

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

    // start indexing blockchain events
    listener.start().await
}
