use std::sync::Arc;

use alloy_eips::eip2718::Typed2718;
use bera_reth::transaction::POL_TX_TYPE;
use eyre::Result;
use futures::StreamExt;
use reth::api::BlockBody;
use reth::core::primitives::AlloyBlockHeader;
use reth_exex::ExExContext;
use reth_node_api::FullNodeComponents;
use reth_primitives_traits::SignedTransaction;
use reth_tracing::tracing::info;
use tokio_postgres::Client;

pub async fn my_indexer<Node: FullNodeComponents>(
    mut ctx: ExExContext<Node>,
    client: Arc<Client>,
) -> Result<()> {
    while let Some(Ok(notification)) = ctx.notifications.next().await {
        // We ignore ChainReorged and ChainReverted since Berachain has fast finality via CometBFT.
        if let Some(committed) = notification.committed_chain() {
            for (block, receipts) in committed.blocks_and_receipts() {
                info!(
                    "Processing block {} with {} transactions",
                    block.number(),
                    block.body().transactions().len()
                );

                for (tx, receipt) in block.body().transactions_iter().zip(receipts.iter()) {
                    // Check if this is a PoL transaction by looking at the tx type
                    if tx.ty() == POL_TX_TYPE {
                        info!("PoL TX {}: {:?}", tx.tx_hash(), receipt);
                    }
                }
                ctx.send_finished_height(block.num_hash())?;
            }
        }
    }

    Ok(())
}
