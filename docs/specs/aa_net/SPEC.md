# aa_net — Subsystem Specification

> **Normative** | Priority P2

## UE5 Reference
- `NetDriver.h`, `ReplicationGraph.h`, `Iris/ReplicationSystem.h`
- `Samples/Games/Lyra/.../LyraReplicationGraph.cpp`

## Requirements

| ID | Requirement |
|----|-------------|
| REQ-NET-001 | Server tick MUST be 60 Hz |
| REQ-NET-002 | `NetEntityId` MUST map 1:1 to entity for entity lifetime |
| REQ-NET-003 | Components in `replication.toml` MUST sync server → clients |
| REQ-NET-004 | Spatial relevancy MUST use grid; default radius 80m for `Pawn` |
| REQ-NET-005 | `PlayerState`, `GameState` MUST be always-relevant |
| REQ-NET-006 | Bandwidth p95 MUST ≤ 128 kbps per client @ 8 players |
| REQ-NET-007 | Owning client MUST predict locomotion; server sends corrections |
| REQ-NET-008 | Simulated proxies MUST interpolate, not predict |
| REQ-NET-009 | RPC events MUST be typed enums; server MUST validate all client RPCs |
| REQ-NET-010 | Disconnect MUST cleanup `NetEntityId` mapping |

## API Contract

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct NetEntityId(pub u64);

#[derive(Component)]
pub struct Replicated;

pub trait Replicate: Component {
    fn encode(&self) -> Bytes;
    fn decode(bytes: &Bytes) -> Result<Self, NetError>;
}

pub struct RelevancyGraph {
    pub fn compute_for_connection(&self, conn: ConnectionId) -> HashSet<NetEntityId>;
}

#[derive(Event)]
pub struct ServerRpc<E>(pub E);  // validated server-side only
```

## Test Matrix

| ID | Scenario | Expected | Auto |
|----|----------|----------|------|
| T-NET-01 | 8 clients connect | 0 failed | integration |
| T-NET-02 | Health sync | ±0.1 match server | integration |
| T-NET-03 | Spatial cull | distant pawn not received | integration |
| T-NET-04 | Bandwidth | p95 ≤ 128 kbps | bench |
| T-NET-05 | 100ms RTT | playable locomotion | playtest |

## Acceptance: P2 when T-NET-01–05 green + Gate P2 PASS.
