use std::sync::Arc;
use std::time::Duration;

use bigdecimal::{BigDecimal, FromPrimitive};
use ethers::providers::{Middleware, Provider, Ws};

use crate::adapters::Adapter;
use crate::listeners::listener::{NoAdapter, SomeAdapter};

pub struct EthereumListener<L> {
    adapter: L,
    contract_addresses: Vec<String>,
    starting_block: Option<usize>,
    rpc_url: String,
    reorg_threshold: BigDecimal,
    chain_id: BigDecimal,

    _marker: std::marker::PhantomData<L>,
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
            _marker: std::marker::PhantomData,
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
            _marker: std::marker::PhantomData,
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
}

impl<A: Adapter> EthereumListener<SomeAdapter<A>> {
    /// Starts listening to the provider
    pub async fn start(self) {
        let provider = Provider::<Ws>::connect(self.rpc_url.clone())
            .await
            .expect("Could not connect to RPC")
            .interval(Duration::from_secs(2));
        let client = Arc::new(provider);
        // let contract = Racer::new(
        //     self.contract_address
        //         .parse::<H160>()
        //         .expect("Invalid contract address"),
        //     client.clone(),
        // );
        let chain_id = client.get_chainid().await.unwrap();

        tracing::info!("Connected to chain {}", chain_id);

        // self.chain_id = bytes_to_bigdecimal(chain_id);
        // self.listen_blocks(client, &contract).await;
    }
}
