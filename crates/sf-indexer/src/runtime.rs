use std::sync::Arc;
use eyre::{Result, Report};
use jsonrpsee::http_client::{HeaderMap, HeaderValue, HttpClient, HttpClientBuilder};
use backoff::{
  future::retry, ExponentialBackoff,
};
use sui_json_rpc::{JsonRpcServerBuilder, ServerHandle, CLIENT_SDK_TYPE_HEADER};
use sui_core::event_handler::SubscriptionHandler;
use crate::checkpoint_handler::CheckpointHandler;

pub struct FirehoseStreamer {
  pub current_block_height: u64,
  checkpoint_handler: Option<CheckpointHandler>,
}

impl FirehoseStreamer {
  pub fn new(starting_block: u64,) -> Self {
    Self {
      current_block_height: starting_block,
      checkpoint_handler: None,
    }
  }

  pub async fn start(&mut self, rpc_client_url: &str) -> Result<()> {
      // Format is FIRE INIT aptos-node <PACKAGE_VERSION> <MAJOR_VERSION> <MINOR_VERSION> <CHAIN_ID>
    println!(
      "\nFIRE INIT sui-node {} sui",
      env!("CARGO_PKG_VERSION"),
    );

    let event_handler = Arc::new(SubscriptionHandler::default());

    let checkpoint_handler = backoff::future::retry(ExponentialBackoff::default(), || async {
      let http_client = get_http_client(rpc_client_url)?;
      let cp = CheckpointHandler::new(http_client, Arc::clone(&event_handler));

      Ok(cp)
    }).await?;

    self.checkpoint_handler = Some(checkpoint_handler);

    loop {
      self.convert_next_block().await;
    }
  }

  pub async fn convert_next_block(&mut self) -> Vec<()> {
    let checkpoint_handler = self.checkpoint_handler.as_ref().expect("Checkpoint handler should be created");
    todo!()
  }
}

fn get_http_client(rpc_client_url: &str) -> Result<HttpClient> {
  let mut headers = HeaderMap::new();
  headers.insert(CLIENT_SDK_TYPE_HEADER, HeaderValue::from_static("indexer"));

  HttpClientBuilder::default()
  .max_request_body_size(2 << 30)
  .max_concurrent_requests(usize::MAX)
  .set_headers(headers.clone())
  .build(rpc_client_url)
  .map_err(|e| {
    Report::msg(format!("Failed to initialize fullnode RPC client with error: {:?}", e))
  })
}
