//! Bera-Reth ExEx Template

mod exex;
mod datasets;
mod indexer;
mod table_definitions;
mod db_writer;
mod utils;

use bera_reth::{
    chainspec::BerachainChainSpec,
    consensus::BerachainBeaconConsensus,
    node::BerachainNode,
    node::evm::config::BerachainEvmConfig,
};
use bera_reth::chainspec::BerachainChainSpecParser;
use clap::Parser;
use eyre::{Result, WrapErr};
use futures::{TryStreamExt};
use indexer::{Indexer, EthereumBlockData};
use reth_cli_commands::node::NoArgs;
use reth_evm::EthEvmFactory;
use reth_exex::{ExExEvent, ExExNotification};
use reth_node_api::FullNodeComponents;
use reth_node_builder::NodeHandle;
use reth_tracing::tracing::{info, warn};
use reth::{CliRunner, cli::Cli};
use std::sync::Arc;
use tokio::sync::oneshot;
use utils::{Config, connect_to_postgres, create_tables};

fn main() -> Result<()> {
    let cli_components_builder = |spec: Arc<BerachainChainSpec>| {
        (
            BerachainEvmConfig::new_with_evm_factory(spec.clone(), EthEvmFactory::default()),
            BerachainBeaconConsensus::new(spec),
        )
    };

    if let Err(err) = Cli::<BerachainChainSpecParser, NoArgs>::parse()
        .with_runner_and_components::<BerachainNode>(
            CliRunner::try_default_runtime().expect("Failed to create default runtime"),
            cli_components_builder,
            async move |builder, _| {
                // Load configs from yaml file
                let config = Config::load().wrap_err("Failed to load configuration")?;

                // Initialize postgres database client
                let client = Arc::new(connect_to_postgres().await?);
                create_tables(&client).await?;
                
                // Create indexer with all processors initialized internally
                let indexer = Indexer::new(config);

                // Create rpc handle channel
                let (rpc_handle_tx, rpc_handle_rx) = oneshot::channel();

                info!(target: "reth::cli", "Launching Berachain ExEx node");
                let NodeHandle {
                    node: _node,
                    node_exit_future,
                } = builder
                    .node(BerachainNode::default())
                    .install_exex("my_indexer", |ctx| async move |ctx| -> Result<()> {
                        // Wait for the ethapi to be sent from the main function
                        let rpc_handle = rpc_handle_rx.await?;
                        info!("Received rpc handle inside exex");

                        // obtain the ethapi from the rpc handle
                        let eth_api = rpc_handle.eth_api();
                        let trace_api = rpc_handle.trace_api();

                        // Process all new chain state notifications
                        while let Some(notification) = ctx.notifications.try_next().await? {
                            match &notification {
                                ExExNotification::ChainReverted { old } => {
                                    let blocks: Vec<_> = old.blocks_iter().collect();
                                    let block_numbers: Vec<i64> = blocks.iter().map(|b| b.num_hash().number as i64).collect();

                                    // Revert all events for the given blocks
                                    if let Err(e) = indexer.revert_blocks(&block_numbers, &client).await {
                                        warn!("Failed to revert blocks: {}", e);
                                    }

                                    info!(block_range = ?old.range(), "Successfully reverted block data");
                                },
                                ExExNotification::ChainCommitted { new } => {
                                    let blocks_and_receipts: Vec<EthereumBlockData> = new.blocks_and_receipts()
                                        .map(|(block, receipts)| (block.clone(), receipts.clone()))
                                        .collect();

                                    // Process the committed blocks
                                    if let Err(e) = indexer.process_blocks(
                                        blocks_and_receipts,
                                        &client,
                                        ctx.provider().clone(),
                                        eth_api,
                                        &trace_api
                                    ).await {
                                        warn!("Failed to process committed blocks: {}", e);
                                    }

                                    // Advance ExEx
                                    ctx.events.send(ExExEvent::FinishedHeight(new.tip().num_hash()))?;
                                },
                                ExExNotification::ChainReorged { old, new } => {
                                    info!(from_chain = ?old.range(), to_chain = ?new.range(), "Received reorg");
                                },
                            }
                        }

                        Ok(())
                    })
                    .launch()
                    .await?;

                // Retrieve the rpc handle from the node and send it to the exex
                rpc_handle_tx
                    .send(_node.add_ons_handle.clone())
                    .expect("Failed to send ethapi to ExEx");

                node_exit_future.await
            },
        )
    {
        eprintln!("Error: {err:?}");
        std::process::exit(1);
    }

    Ok(())
}
