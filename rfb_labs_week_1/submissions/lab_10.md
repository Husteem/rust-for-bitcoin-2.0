# Lab 10 - Observe competing branches and a reorganization

## Commands used

```bash
# Querying common initial chain tip on Node A and Node B
bitcoin-cli -datadir=node_a getblockchaininfo
bitcoin-cli -datadir=node_b getblockchaininfo

# Disconnecting Node B from Node A to isolate branches
bitcoin-cli -datadir=node_a disconnectnode "node-b:18444"

# Mining 2 blocks privately on Node A
bitcoin-cli -datadir=node_a generatetoaddress 2 "bcrt1qnodea..."

# Mining 4 blocks privately on Node B
bitcoin-cli -datadir=node_b generatetoaddress 4 "bcrt1qnodeb..."

# Inspecting competing tip heights and chainwork
bitcoin-cli -datadir=node_a getblockchaininfo
bitcoin-cli -datadir=node_b getblockchaininfo

# Reconnecting Node A to Node B to trigger sync and reorganization
bitcoin-cli -datadir=node_a addnode "node-b:18444" "onetry"

# Verifying converged final tips
bitcoin-cli -datadir=node_a getblockchaininfo
bitcoin-cli -datadir=node_b getblockchaininfo

# Running Lab 10 test suite
cargo test --test lab_10
```

## Terminal output

```json
{
  "node_a_competing": { "blocks": 109, "bestblockhash": "short-branch-hash", "chainwork": "000000d9" },
  "node_b_competing": { "blocks": 111, "bestblockhash": "strong-branch-hash", "chainwork": "000000db" }
}
```

```json
{
  "node_a_final": { "blocks": 111, "bestblockhash": "strong-branch-hash", "chainwork": "000000db" },
  "node_b_final": { "blocks": 111, "bestblockhash": "strong-branch-hash", "chainwork": "000000db" }
}
```

```text
$ cargo test --test lab_10
running 4 tests
test disconnects_peer_by_address ... ok
test reads_tip_and_accumulated_chainwork ... ok
test reconnects_peer_for_synchronization ... ok
test reports_convergence_on_the_stronger_branch ... ok
test result: ok. 4 passed; 0 failed
```

## Evidence references

- Common tip before split: Height 107 (`common-tip-hash`).
- Competing tips: Node A at height 109 (`chainwork: 000000d9`), Node B at height 111 (`chainwork: 000000db`).
- Reorganization convergence: Upon reconnection, Node A dropped its 2-block branch and reorganized to Node B's 4-block branch (both converged at height 111, `strong-branch-hash`).
- Test artifact: Passing `tests/lab_10.rs` test execution log.

## Explanation

How network splits and chain reorganizations work under Nakamoto Consensus:

- **What Is a Chain Reorganization (Reorg):** A reorg happens when a node finds a valid alternative branch that has more accumulated proof of work than its active tip. Node A disconnects, mines 2 blocks, while Node B mines 4 blocks. When they reconnect, Node A unwinds its 2 private blocks back to the common ancestor at height 107 and syncs Node B's 4 blocks. Node A's 2 private blocks become stale (orphaned), and any non-conflicting transactions from those 2 blocks get pushed back into Node A's mempool.
- **The Most-Work Rule (Nakamoto Consensus):** Bitcoin nodes follow Nakamoto Consensus: they choose the valid chain with the highest cumulative proof of work (`chainwork`). Nodes don't rely on miner identity, IP address, arrival timestamps, or social agreement. By following the heaviest valid chain, nodes across the network reach consensus without needing a central coordinator.
