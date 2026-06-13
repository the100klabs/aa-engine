# 03 — Platform Matrix

> **Normative.** P0 MUST ship; P1 SHOULD ship for AA; P2 MAY ship post-AA.

## Client

| Platform | Arch | P0 | P1 (AA) | P2 | Notes |
|----------|------|----|---------|----|-------|
| Windows | x86_64 | MUST | MUST | — | Primary ship target |
| macOS | aarch64 | MUST | MUST | — | Dev + ship |
| macOS | x86_64 | — | SHOULD | — | Rosetta acceptable |
| Linux | x86_64 | — | SHOULD | MUST | Steam Deck class |
| Web (WASM) | — | — | — | MAY | Not AA |

## Server

| Platform | Arch | P0 | P1 (AA) | Notes |
|----------|------|----|---------|-------|
| Linux | x86_64 | — | MUST | Dedicated server |
| Windows | x86_64 | — | SHOULD | LAN host |
| macOS | aarch64 | — | — | Dev only |

## Editor / Studio

| Platform | P0 | P1 (AA) |
|----------|----|---------|
| macOS aarch64 | MUST | MUST |
| Windows x86_64 | SHOULD | MUST |
| Linux | — | SHOULD |

## GPU / RHI (via wgpu)

| Backend | P0 | P1 (AA) |
|---------|----|---------|
| Vulkan | SHOULD | MUST |
| Metal | MUST | MUST |
| DX12 | SHOULD | MUST |
| WebGPU | — | — |

## Input Devices

| Device | P0 | P1 (AA) |
|--------|----|---------|
| Keyboard + mouse | MUST | MUST |
| Xbox-style gamepad | SHOULD | MUST |
| PlayStation gamepad | — | SHOULD |

## Build Targets

| Binary | Windows | macOS | Linux server |
|--------|---------|-------|--------------|
| `aa_game` | MUST | MUST | — |
| `aa_server` | SHOULD | — | MUST |
| `aa_editor` | SHOULD | MUST | — |
| `aa_cli` | MUST | MUST | MUST |

## Feature Flags by Platform

| Feature | Desktop client | Dedicated server | Editor |
|---------|----------------|------------------|--------|
| Rendering | MUST | MUST NOT | MUST |
| Audio | MUST | MUST NOT | SHOULD |
| Networking | MUST | MUST | SHOULD |
| `aa_editor` UI | MUST NOT | MUST NOT | MUST |
| `aa_agent` | — | — | MUST |

## Minimum Specs (AA ship label)

| | Client | Server |
|---|--------|--------|
| CPU | 6 cores / 12 threads | 4 vCPU |
| GPU | 6 GB VRAM | — |
| RAM | 16 GB | 8 GB |
| Storage | 20 GB | 10 GB |
| Network | 5 Mbps | 100 Mbps upstream |

## CI Matrix

| Job | Platform | Required on PR |
|-----|----------|----------------|
| `check + test` | Linux x64 | MUST |
| `check + test` | Windows | MUST (nightly if slow) |
| `check + test` | macOS aarch64 | MUST (nightly if slow) |
| `aa validate` | Linux | MUST |
| `playtest smoke` | Linux headless | MUST |
| Perf regression | Linux AA Target | Weekly |
