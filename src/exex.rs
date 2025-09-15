use futures::StreamExt;
use reth::api::BlockBody;
use reth::core::primitives::AlloyBlockHeader;
use reth_exex::ExExContext;
use reth_node_api::FullNodeComponents;
use reth_tracing::tracing::info;

pub async fn my_indexer<Node: FullNodeComponents>(mut ctx: ExExContext<Node>) -> eyre::Result<()> {
    while let Some(Ok(notification)) = ctx.notifications.next().await {
        // We ignore ChainReorged and ChainReverted since Berachain has fast finality via CometBFT.
        if let Some(committed) = notification.committed_chain() {
            for (block, _receipts) in committed.blocks_and_receipts() {
                info!(
                    "Processing block {} with {} transactions",
                    block.number(),
                    block.body().transactions().len()
                );
                ctx.send_finished_height(block.num_hash())?;
            }
        }
    }

    Ok(())
}
