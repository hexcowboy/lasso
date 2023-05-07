use std::sync::Arc;
use std::time::Duration;

use bigdecimal::{BigDecimal, FromPrimitive};
use ethers::providers::{Middleware, Provider, StreamExt, Ws};
use tokio::select;
use tokio::sync::watch;

use crate::adapters::Adapter;
use crate::listeners::listener::{NoAdapter, SomeAdapter};

pub struct EthereumListener<A> {
    adapter: A,
    contract_addresses: Vec<String>,
    starting_block: Option<usize>,
    rpc_url: String,
    reorg_threshold: BigDecimal,
    chain_id: BigDecimal,
    time_between_indexes: Option<Duration>,
    killswitch: watch::Receiver<()>,
}

impl EthereumListener<NoAdapter> {
    pub fn new() -> Self {
        EthereumListener {
            adapter: NoAdapter,
            contract_addresses: Vec::new(),
            starting_block: None,
            rpc_url: String::new(),
            reorg_threshold: BigDecimal::from(7),
            chain_id: BigDecimal::from(1),
            time_between_indexes: None,
            killswitch: watch::channel(()).1,
        }
    }
}

impl<T> EthereumListener<T> {
    pub fn with_adapter<A: Adapter>(self, adapter: A) -> EthereumListener<SomeAdapter<A>> {
        EthereumListener {
            adapter: SomeAdapter(Arc::new(adapter)),
            contract_addresses: self.contract_addresses,
            starting_block: self.starting_block,
            rpc_url: self.rpc_url,
            reorg_threshold: self.reorg_threshold,
            chain_id: self.chain_id,
            time_between_indexes: self.time_between_indexes,
            killswitch: self.killswitch,
        }
    }

    pub fn with_starting_block(mut self, starting_block: usize) -> Self {
        self.starting_block = Some(starting_block);
        self
    }

    pub fn with_rpc_url(mut self, rpc_url: String) -> Self {
        self.rpc_url = rpc_url;
        self
    }

    pub fn with_reorg_threshold(mut self, threshold: usize) -> Self {
        self.reorg_threshold = BigDecimal::from_usize(threshold).unwrap_or({
            tracing::warn!("Invalid reorg threshold, defaulting to 7");
            BigDecimal::from(7)
        });
        self
    }

    pub fn with_time_between_indexes(mut self, time: Duration) -> Self {
        self.time_between_indexes = Some(time);
        self
    }

    pub fn with_killswitch(mut self, killswitch: watch::Receiver<()>) -> Self {
        self.killswitch = killswitch;
        self
    }
}

impl<A: Adapter> EthereumListener<SomeAdapter<A>> {
    /// Starts listening to the provider
    pub async fn start(mut self) {
        let provider = Provider::<Ws>::connect(self.rpc_url.clone())
            .await
            .expect("Could not connect to RPC")
            .interval(Duration::from_secs(2));
        let client = provider;
        let chain_id = client.get_chainid().await.unwrap();

        tracing::info!("Connected to EVM chain {}", chain_id);

        match self.time_between_indexes {
            Some(time) => {
                tracing::info!("Indexing every {:?} (paced mode)", time);
                self.poll_blocks(client, time).await;
            }
            None => {
                tracing::info!("Indexing every block (realtime mode)");
                self.subscribe_blocks(client).await;
            }
        }
    }

    pub async fn poll_blocks(&mut self, client: Provider<Ws>, interval: Duration) {
        let mut interval = tokio::time::interval(interval);

        loop {
            select! {
                _ = interval.tick() => {
                    let block = client.get_block_number().await.unwrap();
                    tracing::info!("New block {}", block);
                }
                _ = self.killswitch.changed() => {
                    tracing::warn!("Killswitch triggered, shutting down");
                    break;
                }
            }
        }
    }

    pub async fn subscribe_blocks(&mut self, client: Provider<Ws>) {
        let mut stream = client
            .subscribe_blocks()
            .await
            .expect("Could not subscribe to new blocks");

        loop {
            select! {
                block = stream.next() => {
                    tracing::info!("Found block {:?}", block);
                }
                _ = self.killswitch.changed() => {
                    tracing::warn!("Killswitch triggered, shutting down");
                    break;
                }
            }
        }
    }
}
