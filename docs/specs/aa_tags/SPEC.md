# aa_tags — Subsystem Specification

> **Normative** | Priority P1

## Requirements

| ID | Requirement |
|----|-------------|
| REQ-TAG-001 | Tag dictionary MUST load from `assets/data/tags.ron` |
| REQ-TAG-002 | `TagId` MUST be interned u32; lookup O(1) |
| REQ-TAG-003 | Hierarchy `A.B.C` MUST imply parent paths for query |
| REQ-TAG-004 | `TagQuery::HasAll` / `HasAny` / `HasNone` MUST match UE semantics |
| REQ-TAG-005 | Unknown tag in asset MUST fail validation |

## API Contract

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TagId(pub u32);

pub struct TagRegistry { /* load dictionary */ }

pub struct TagContainer(BitSet);  // or equivalent

pub enum TagQuery {
    HasAll(Vec<TagId>),
    HasAny(Vec<TagId>),
    HasNone(Vec<TagId>),
}

impl TagContainer {
    pub fn matches(&self, query: &TagQuery) -> bool;
}
```

## Test Matrix

| ID | Scenario | Expected | Auto |
|----|----------|----------|------|
| T-TAG-01 | HasAll | correct | unit |
| T-TAG-02 | Parent match | A.B matches query A | unit |
| T-TAG-03 | Unknown tag validate | error | integration |

## Acceptance: P1 when T-TAG-01–03 green.
