use alloy::{providers::Provider, rpc::types::Filter};
use kohaku_rpc_provider::{StreamedEvent, StreamedEventConfig};
use kohaku_tornadocash::{
    Pool,
    event::{LogDecodeError, Snapshot, SyncEvent},
};
use tracing::{info, warn};

use crate::{Syncer, SyncerError};

/// A syncer that reads from an Ethereum JSON-RPC provider
#[derive(Clone)]
pub struct RpcSyncer<P: Provider> {
    provider: P,
    config: StreamedEventConfig,
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
enum RpcSyncerError {
    #[error("Error decoding log: {0}")]
    LogDecodeError(#[from] LogDecodeError),
    #[error("RPC error: {0}")]
    RpcError(#[from] alloy::transports::RpcError<alloy::transports::TransportErrorKind>),
    #[error("Snapshot error: {0}")]
    Snapshot(#[from] kohaku_tornadocash::event::SnapshotError),
}

impl<P: Provider> RpcSyncer<P> {
    pub fn new(provider: P) -> Self {
        Self {
            provider,
            config: StreamedEventConfig::default(),
        }
    }

    /// Sets the chunking, concurrency and backoff options for `eth_getLogs` calls.
    #[must_use]
    pub fn with_config(mut self, config: StreamedEventConfig) -> Self {
        self.config = config;
        self
    }
}

#[cfg_attr(native, async_trait::async_trait)]
#[cfg_attr(wasm, async_trait::async_trait(?Send))]
impl<P: Provider + Clone> Syncer for RpcSyncer<P> {
    async fn sync_range(
        &self,
        pools: &[Pool],
        from_block: u64,
        to_block: u64,
    ) -> Result<Vec<Snapshot>, SyncerError> {
        Ok(self
            .sync(pools, from_block, to_block)
            .await
            .map_err(SyncerError::other)?)
    }
}

impl<P: Provider + Clone> RpcSyncer<P> {
    /// Reads every pool's logs in one scan, then routes each log to its pools.
    async fn sync(
        &self,
        pools: &[Pool],
        from_block: u64,
        to_block: u64,
    ) -> Result<Vec<Snapshot>, RpcSyncerError> {
        //? `latest_block` is an inclusive block number, `to_block` an exclusive bound.
        let latest = self.provider.get_block_number().await?.saturating_add(1);
        let to = to_block.min(latest);
        let ranges: Vec<_> = pools
            .iter()
            .map(|pool| {
                let from = from_block.max(pool.deployed_block);
                from..to.max(from)
            })
            .collect();
        let mut events = vec![Vec::new(); pools.len()];

        let scan_start = ranges
            .iter()
            .filter(|range| !range.is_empty())
            .map(|range| range.start)
            .min();
        if let Some(scan_start) = scan_start {
            info!("Syncing from {} to {}", scan_start, to);

            //? `eth_getLogs` treats both block bounds as inclusive
            let filter = Filter::new()
                .address(pools.iter().map(|pool| pool.address).collect::<Vec<_>>())
                .from_block(scan_start)
                .to_block(to - 1);
            let logs = StreamedEvent::new(self.provider.clone(), filter)
                .config(self.config)
                .query()
                .await?;

            for log in logs {
                let event = match SyncEvent::try_from(&log) {
                    Ok(event) => event,
                    Err(LogDecodeError::UnknownEvent) => {
                        warn!("Unknown event with topic {:?}", log.topic0());
                        continue;
                    }
                    Err(e) => return Err(e.into()),
                };
                let block = match &event {
                    SyncEvent::Deposit(deposit) => deposit.block_number,
                    SyncEvent::Withdrawal(withdrawal) => withdrawal.block_number,
                };

                for ((pool, range), pool_events) in pools.iter().zip(&ranges).zip(&mut events) {
                    //? A pool's `deployed_block` can be later than the scan start, and
                    //? `Snapshot::new` rejects events outside the pool's range.
                    if pool.address == log.address() && range.contains(&block) {
                        pool_events.push(event.clone());
                    }
                }
            }
        }

        pools
            .iter()
            .zip(ranges)
            .zip(events)
            .map(|((pool, range), events)| {
                info!("Synced {} events for {}", events.len(), pool);
                Ok(Snapshot::new(pool.clone(), range, events)?)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use alloy::{
        primitives::{Address, B256, LogData, U256},
        providers::ProviderBuilder,
        rpc::types::Log,
        sol,
        sol_types::SolEvent,
        transports::mock::Asserter,
    };
    use kohaku_tornadocash::event::{DepositEvent, WithdrawalEvent};

    use super::*;

    sol!(
        event Deposit(bytes32 indexed commitment, uint32 leafIndex, uint256 timestamp);
        event Withdrawal(address to, bytes32 nullifierHash, address indexed relayer, uint256 fee);
    );

    #[tokio::test]
    async fn syncs_events_within_clamped_range() {
        let pool = Pool::SEPOLIA_ETHER_01;
        let deposit_block = pool.deployed_block + 3;
        let latest = pool.deployed_block + 9;
        let commitment = B256::with_last_byte(1);
        let nullifier_hash = B256::with_last_byte(2);
        let to = Address::with_last_byte(3);
        let relayer = Address::with_last_byte(4);
        let fee = U256::from(5);

        let deposit = Deposit {
            commitment,
            leafIndex: 0,
            timestamp: U256::ZERO,
        };
        let withdrawal = Withdrawal {
            to,
            nullifierHash: nullifier_hash,
            relayer,
            fee,
        };

        let asserter = Asserter::new();
        asserter.push_success(&latest);
        asserter.push_success(&vec![
            log_at(&pool, deposit.encode_log_data(), deposit_block),
            log_at(&pool, withdrawal.encode_log_data(), latest),
        ]);
        let provider = ProviderBuilder::new().connect_mocked_client(asserter.clone());

        let [snapshot] = RpcSyncer::new(provider)
            .sync_range(std::slice::from_ref(&pool), 0, u64::MAX)
            .await
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(snapshot.range(), pool.deployed_block..latest + 1);
        assert_eq!(
            snapshot.events(),
            &[
                SyncEvent::Deposit(DepositEvent {
                    commitment: commitment.try_into().unwrap(),
                    leaf_index: 0,
                    block_number: deposit_block,
                }),
                SyncEvent::Withdrawal(WithdrawalEvent {
                    to,
                    nullifier_hash: nullifier_hash.try_into().unwrap(),
                    relayer,
                    fee,
                    block_number: latest,
                }),
            ]
        );
        assert!(asserter.read_q().is_empty());
    }

    #[tokio::test]
    async fn splits_one_scan_across_pools() {
        let first = Pool::SEPOLIA_ETHER_01;
        let mut second = Pool::SEPOLIA_ETHER_01;
        second.address = Address::with_last_byte(9);
        second.deployed_block = first.deployed_block + 2;
        let latest = first.deployed_block + 9;
        let deposit = |leaf_index| Deposit {
            commitment: B256::with_last_byte(1),
            leafIndex: leaf_index,
            timestamp: U256::ZERO,
        };

        let asserter = Asserter::new();
        asserter.push_success(&latest);
        asserter.push_success(&vec![
            log_at(&first, deposit(0).encode_log_data(), first.deployed_block),
            log_at(&second, deposit(0).encode_log_data(), second.deployed_block),
            log_at(&first, deposit(1).encode_log_data(), latest),
        ]);
        let provider = ProviderBuilder::new().connect_mocked_client(asserter.clone());

        let [first_snapshot, second_snapshot] = RpcSyncer::new(provider)
            .sync_range(&[first.clone(), second.clone()], 0, u64::MAX)
            .await
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(first_snapshot.range(), first.deployed_block..latest + 1);
        assert_eq!(first_snapshot.events().len(), 2);
        assert_eq!(second_snapshot.range(), second.deployed_block..latest + 1);
        assert_eq!(second_snapshot.events().len(), 1);
        assert!(asserter.read_q().is_empty());
    }

    fn log_at(pool: &Pool, data: LogData, block: u64) -> Log {
        Log {
            inner: alloy::primitives::Log {
                address: pool.address,
                data,
            },
            block_number: Some(block),
            ..Default::default()
        }
    }
}
