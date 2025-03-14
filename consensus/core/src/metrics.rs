// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use crate::network::metrics::{NetworkRouteMetrics, QuinnConnectionMetrics};
use prometheus::{
    register_histogram_vec_with_registry, register_histogram_with_registry,
    register_int_counter_vec_with_registry, register_int_counter_with_registry,
    register_int_gauge_vec_with_registry, register_int_gauge_with_registry, Histogram,
    HistogramVec, IntCounter, IntCounterVec, IntGauge, IntGaugeVec, Registry,
};

// starts from 1μs, 50μs, 100μs...
const FINE_GRAINED_LATENCY_SEC_BUCKETS: &[f64] = &[
    0.000_001, 0.000_050, 0.000_100, 0.000_500, 0.001, 0.005, 0.01, 0.05, 0.1, 0.15, 0.2, 0.25,
    0.3, 0.35, 0.4, 0.45, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.2, 1.4, 1.6, 1.8, 2.0, 2.5, 3.0, 3.5,
    4.0, 4.5, 5.0, 5.5, 6.0, 6.5, 7.0, 7.5, 8.0, 8.5, 9.0, 9.5, 10.,
];

const LATENCY_SEC_BUCKETS: &[f64] = &[
    0.001, 0.005, 0.01, 0.05, 0.1, 0.15, 0.2, 0.25, 0.3, 0.35, 0.4, 0.45, 0.5, 0.6, 0.7, 0.8, 0.9,
    1.0, 1.2, 1.4, 1.6, 1.8, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0, 5.5, 6.0, 6.5, 7.0, 7.5, 8.0, 8.5,
    9.0, 9.5, 10., 12.5, 15., 17.5, 20., 25., 30., 60., 90., 120., 180., 300.,
];

const SIZE_BUCKETS: &[f64] = &[
    100.,
    400.,
    800.,
    1_000.,
    2_000.,
    5_000.,
    10_000.,
    20_000.,
    50_000.,
    100_000.,
    200_000.0,
    300_000.0,
    400_000.0,
    500_000.0,
    1_000_000.0,
    2_000_000.0,
    3_000_000.0,
    5_000_000.0,
    10_000_000.0,
]; // size in bytes

pub(crate) struct Metrics {
    pub(crate) node_metrics: NodeMetrics,
    pub(crate) channel_metrics: ChannelMetrics,
    pub(crate) network_metrics: NetworkMetrics,
    pub(crate) quinn_connection_metrics: QuinnConnectionMetrics,
}

pub(crate) fn initialise_metrics(registry: Registry) -> Arc<Metrics> {
    let node_metrics = NodeMetrics::new(&registry);
    let channel_metrics = ChannelMetrics::new(&registry);
    let network_metrics = NetworkMetrics::new(&registry);
    let quinn_connection_metrics = QuinnConnectionMetrics::new(&registry);

    Arc::new(Metrics {
        node_metrics,
        channel_metrics,
        network_metrics,
        quinn_connection_metrics,
    })
}

#[cfg(test)]
pub(crate) fn test_metrics() -> Arc<Metrics> {
    initialise_metrics(Registry::new())
}

pub(crate) struct NodeMetrics {
    pub block_commit_latency: Histogram,
    pub block_proposed: IntCounterVec,
    pub block_size: Histogram,
    pub block_timestamp_drift_wait_ms: IntCounterVec,
    pub blocks_per_commit_count: Histogram,
    pub broadcaster_rtt_estimate_ms: IntGaugeVec,
    pub core_lock_dequeued: IntCounter,
    pub core_lock_enqueued: IntCounter,
    pub highest_accepted_round: IntGauge,
    pub accepted_blocks: IntCounter,
    pub dag_state_recent_blocks: IntGauge,
    pub dag_state_recent_refs: IntGauge,
    pub dag_state_store_read_count: IntCounterVec,
    pub dag_state_store_write_count: IntCounter,
    pub fetch_blocks_scheduler_inflight: IntGauge,
    pub fetched_blocks: IntCounterVec,
    pub invalid_blocks: IntCounterVec,
    pub rejected_future_blocks: IntCounterVec,
    pub verified_blocks: IntCounterVec,
    pub committed_leaders_total: IntCounterVec,
    pub last_committed_leader_round: IntGauge,
    pub commit_round_advancement_interval: Histogram,
    pub last_decided_leader_round: IntGauge,
    pub leader_timeout_total: IntCounter,
    pub missing_blocks_total: IntGauge,
    pub quorum_receive_latency: Histogram,
    pub scope_processing_time: HistogramVec,
    pub sub_dags_per_commit_count: Histogram,
    pub block_suspensions: IntCounterVec,
    pub block_unsuspensions: IntCounterVec,
    pub block_manager_suspended_blocks: IntGauge,
    pub block_manager_missing_ancestors: IntGauge,
    pub block_manager_missing_blocks: IntGauge,
    pub threshold_clock_round: IntGauge,
    pub subscriber_connection_attempts: IntCounterVec,
    pub subscriber_connections: IntGaugeVec,
    pub commit_sync_inflight_fetches: IntGauge,
    pub commit_sync_pending_fetches: IntGauge,
    pub commit_sync_fetched_commits: IntCounter,
    pub commit_sync_fetched_blocks: IntCounter,
    pub commit_sync_total_fetched_blocks_size: IntCounter,
    pub commit_sync_local_index: IntGauge,
    pub commit_sync_fetch_loop_latency: Histogram,
    pub commit_sync_fetch_once_latency: Histogram,
    pub uptime: Histogram,
}

impl NodeMetrics {
    pub fn new(registry: &Registry) -> Self {
        Self {
            block_commit_latency: register_histogram_with_registry!(
                "block_commit_latency",
                "The time taken between block creation and block commit.",
                registry,
            ).unwrap(),
            block_proposed: register_int_counter_vec_with_registry!(
                "block_proposed",
                "Total number of block proposals. If force is true then this block has been created forcefully via a leader timeout event.",
                &["force"],
                registry,
            ).unwrap(),
            block_size: register_histogram_with_registry!(
                "block_size",
                "The size (in bytes) of proposed blocks",
                SIZE_BUCKETS.to_vec(),
                registry
            ).unwrap(),
            block_timestamp_drift_wait_ms: register_int_counter_vec_with_registry!(
                "block_timestamp_drift_wait_ms",
                "Total time in ms spent waiting, when a received block has timestamp in future.",
                &["authority"],
                registry,
            ).unwrap(),
            blocks_per_commit_count: register_histogram_with_registry!(
                "blocks_per_commit_count",
                "The number of blocks per commit.",
                registry,
            ).unwrap(),
            broadcaster_rtt_estimate_ms: register_int_gauge_vec_with_registry!(
                "broadcaster_rtt_estimate_ms",
                "Estimated RTT latency per peer authority, for block sending in Broadcaster",
                &["peer"],
                registry,
            ).unwrap(),
            core_lock_dequeued: register_int_counter_with_registry!(
                "core_lock_dequeued",
                "Number of dequeued core requests",
                registry,
            ).unwrap(),
            core_lock_enqueued: register_int_counter_with_registry!(
                "core_lock_enqueued",
                "Number of enqueued core requests",
                registry,
            ).unwrap(),
            highest_accepted_round: register_int_gauge_with_registry!(
                "highest_accepted_round",
                "The highest round where a block has been accepted. Resets on restart.",
                registry,
            ).unwrap(),
            accepted_blocks: register_int_counter_with_registry!(
                "accepted_blocks",
                "Number of accepted blocks",
                registry,
            ).unwrap(),
            dag_state_recent_blocks: register_int_gauge_with_registry!(
                "dag_state_recent_blocks",
                "Number of recent blocks cached in the DagState",
                registry,
            ).unwrap(),
            dag_state_recent_refs: register_int_gauge_with_registry!(
                "dag_state_recent_refs",
                "Number of recent refs cached in the DagState",
                registry,
            ).unwrap(),
            dag_state_store_read_count: register_int_counter_vec_with_registry!(
                "dag_state_store_read_count",
                "Number of times DagState needs to read from store per operation type",
                &["type"],
                registry,
            ).unwrap(),
            dag_state_store_write_count: register_int_counter_with_registry!(
                "dag_state_store_write_count",
                "Number of times DagState needs to write to store",
                registry,
            ).unwrap(),
            fetch_blocks_scheduler_inflight: register_int_gauge_with_registry!(
                "fetch_blocks_scheduler_inflight",
                "Designates whether the synchronizer scheduler task to fetch blocks is currently running",
                registry,
            ).unwrap(),
            fetched_blocks: register_int_counter_vec_with_registry!(
                "fetched_blocks",
                "Number of fetched blocks per peer authority via the synchronizer.",
                &["authority", "type"],
                registry,
            ).unwrap(),
            // TODO: add a short status label.
            invalid_blocks: register_int_counter_vec_with_registry!(
                "invalid_blocks",
                "Number of invalid blocks per peer authority",
                &["authority", "source"],
                registry,
            ).unwrap(),
            rejected_future_blocks: register_int_counter_vec_with_registry!(
                "rejected_future_blocks",
                "Number of blocks rejected because their timestamp is too far in the future",
                &["authority"],
                registry,
            ).unwrap(),
            verified_blocks: register_int_counter_vec_with_registry!(
                "verified_blocks",
                "Number of blocks received from each peer that are verified",
                &["authority"],
                registry,
            ).unwrap(),
            committed_leaders_total: register_int_counter_vec_with_registry!(
                "committed_leaders_total",
                "Total number of (direct or indirect) committed leaders per authority",
                &["authority", "commit_type"],
                registry,
            ).unwrap(),
            last_committed_leader_round: register_int_gauge_with_registry!(
                "last_committed_leader_round",
                "The last round where a leader was committed to store and sent to commit consumer.",
                registry,
            ).unwrap(),
            commit_round_advancement_interval: register_histogram_with_registry!(
                "commit_round_advancement_interval",
                "Intervals (in secs) between commit round advancements.",
                FINE_GRAINED_LATENCY_SEC_BUCKETS.to_vec(),
                registry,
            ).unwrap(),
            last_decided_leader_round: register_int_gauge_with_registry!(
                "last_decided_leader_round",
                "The last round where a commit decision was made.",
                registry,
            ).unwrap(),
            leader_timeout_total: register_int_counter_with_registry!(
                "leader_timeout_total",
                "Total number of leader timeouts",
                registry,
            ).unwrap(),
            missing_blocks_total: register_int_gauge_with_registry!(
                "missing_blocks_total",
                "Total number of missing blocks",
                registry,
            ).unwrap(),
            quorum_receive_latency: register_histogram_with_registry!(
                "quorum_receive_latency",
                "The time it took to receive a new round quorum of blocks",
                registry
            ).unwrap(),
            scope_processing_time: register_histogram_vec_with_registry!(
                "scope_processing_time",
                "The processing time of a specific code scope",
                &["scope"],
                FINE_GRAINED_LATENCY_SEC_BUCKETS.to_vec(),
                registry
            ).unwrap(),
            sub_dags_per_commit_count: register_histogram_with_registry!(
                "sub_dags_per_commit_count",
                "The number of subdags per commit.",
                registry,
            ).unwrap(),
            block_suspensions: register_int_counter_vec_with_registry!(
                "block_suspensions",
                "The number block suspensions. The counter is reported uniquely, so if a block is sent for reprocessing while alreadly suspended then is not double counted",
                &["authority"],
                registry,
            ).unwrap(),
            block_unsuspensions: register_int_counter_vec_with_registry!(
                "block_unsuspensions",
                "The number of block unsuspensions.",
                &["authority"],
                registry,
            ).unwrap(),
            block_manager_suspended_blocks: register_int_gauge_with_registry!(
                "block_manager_suspended_blocks",
                "The number of blocks currently suspended in the block manager",
                registry,
            ).unwrap(),
            block_manager_missing_ancestors: register_int_gauge_with_registry!(
                "block_manager_missing_ancestors",
                "The number of missing ancestors tracked in the block manager",
                registry,
            ).unwrap(),
            block_manager_missing_blocks: register_int_gauge_with_registry!(
                "block_manager_missing_blocks",
                "The number of blocks missing content tracked in the block manager",
                registry,
            ).unwrap(),
            threshold_clock_round: register_int_gauge_with_registry!(
                "threshold_clock_round",
                "The current threshold clock round. We only advance to a new round when a quorum of parents have been synced.",
                registry,
            ).unwrap(),
            subscriber_connection_attempts: register_int_counter_vec_with_registry!(
                "subscriber_connection_attempts",
                "The number of connection attempts per peer",
                &["authority", "status"],
                registry,
            ).unwrap(),
            subscriber_connections: register_int_gauge_vec_with_registry!(
                "subscriber_connections",
                "The number of block stream connections breaking down by peer",
                &["authority"],
                registry,
            ).unwrap(),
            commit_sync_inflight_fetches: register_int_gauge_with_registry!(
                "commit_sync_inflight_fetches",
                "The number of inflight fetches in commit syncer",
                registry,
            ).unwrap(),
            commit_sync_pending_fetches: register_int_gauge_with_registry!(
                "commit_sync_pending_fetches",
                "The number of pending fetches in commit syncer",
                registry,
            ).unwrap(),
            commit_sync_fetched_commits: register_int_counter_with_registry!(
                "commit_sync_fetched_commits",
                "The number of commits fetched via commit syncer",
                registry,
            ).unwrap(),
            commit_sync_fetched_blocks: register_int_counter_with_registry!(
                "commit_sync_fetched_blocks",
                "The number of blocks fetched via commit syncer",
                registry,
            ).unwrap(),
            commit_sync_total_fetched_blocks_size: register_int_counter_with_registry!(
                "commit_sync_total_fetched_blocks_size",
                "The total size in bytes of blocks fetched via commit syncer",
                registry,
            ).unwrap(),
            commit_sync_local_index: register_int_gauge_with_registry!(
                "commit_sync_local_index",
                "The max commit index among local and fetched commits",
                registry,
            ).unwrap(),
            commit_sync_fetch_loop_latency: register_histogram_with_registry!(
                "commit_sync_fetch_loop_latency",
                "The time taken to finish fetching commits and blocks from a given range",
                LATENCY_SEC_BUCKETS.to_vec(),
                registry,
            ).unwrap(),
            commit_sync_fetch_once_latency: register_histogram_with_registry!(
                "commit_sync_fetch_once_latency",
                "The time taken to fetch commits and blocks once",
                LATENCY_SEC_BUCKETS.to_vec(),
                registry,
            ).unwrap(),
            uptime: register_histogram_with_registry!(
                "uptime",
                "Total node uptime",
                LATENCY_SEC_BUCKETS.to_vec(),
                registry,
            ).unwrap(),
        }
    }
}

pub(crate) struct ChannelMetrics {
    /// occupancy of the channel from TransactionClient to TransactionConsumer
    pub tx_transactions_submit: IntGauge,
    /// total received on channel from TransactionClient to TransactionConsumer
    pub tx_transactions_submit_total: IntCounter,
    /// occupancy of the CoreThread commands channel
    pub core_thread: IntGauge,
    /// total received on the CoreThread commands channel
    pub core_thread_total: IntCounter,
}

impl ChannelMetrics {
    pub fn new(registry: &Registry) -> Self {
        Self {
            tx_transactions_submit: register_int_gauge_with_registry!(
                "tx_transactions_submit",
                "occupancy of the channel from the `TransactionClient` to the `TransactionConsumer`",
                registry
            ).unwrap(),
            tx_transactions_submit_total: register_int_counter_with_registry!(
                "tx_transactions_submit_total",
                "total received on channel from the `TransactionClient` to the `TransactionConsumer`",
                registry
            ).unwrap(),
            core_thread: register_int_gauge_with_registry!(
                "core_thread",
                "occupancy of the `CoreThread` commands channel",
                registry
            ).unwrap(),
            core_thread_total: register_int_counter_with_registry!(
                "core_thread_total",
                "total received on the `CoreThread` commands channel",
                registry
            ).unwrap(),
        }
    }
}

// Fields for network-agnostic metrics can be added here
pub(crate) struct NetworkMetrics {
    pub network_type: IntGaugeVec,
    pub inbound: NetworkRouteMetrics,
    pub outbound: NetworkRouteMetrics,
}

impl NetworkMetrics {
    pub fn new(registry: &Registry) -> Self {
        Self {
            network_type: register_int_gauge_vec_with_registry!(
                "network_type",
                "Type of the network used: anemo or tonic",
                &["type"],
                registry
            )
            .unwrap(),
            inbound: NetworkRouteMetrics::new("inbound", registry),
            outbound: NetworkRouteMetrics::new("outbound", registry),
        }
    }
}
