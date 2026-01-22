# flux-tui

A minimalist TUI for managing Flux CD resources built with Rust and ratatui.

## Overview

This application provides a terminal interface for viewing and managing Flux CD resources:
- Kustomizations
- HelmReleases
- HelmCharts

## Technology Stack

- **Language**: Rust
- **TUI Framework**: ratatui + crossterm
- **Async Runtime**: tokio
- **K8s Client**: kube-rs (listing) + flux CLI (reconcile)

## Project Structure

```
flux-tui/
├── src/
│   ├── main.rs                 # Entry point, event loop
│   ├── app/
│   │   ├── mod.rs
│   │   ├── state.rs            # Application state (Model)
│   │   ├── actions.rs          # Action enum (Messages)
│   │   └── config.rs           # Configuration
│   ├── kubernetes/
│   │   ├── mod.rs
│   │   ├── client.rs           # kube-rs wrapper
│   │   ├── resources/
│   │   │   ├── mod.rs
│   │   │   ├── kustomization.rs
│   │   │   ├── helmrelease.rs
│   │   │   └── helmchart.rs
│   │   └── reconcile.rs        # flux CLI wrapper
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── draw.rs             # Main view function
│   │   ├── theme.rs            # Minimalist color palette
│   │   ├── layout.rs
│   │   └── widgets/
│   │       ├── mod.rs
│   │       ├── resource_table.rs
│   │       ├── tabs.rs
│   │       └── status_bar.rs
│   └── event/
│       ├── mod.rs
│       └── handler.rs          # Key events → Actions
├── Cargo.toml
├── README.md
└── AGENTS.md
```

## Architecture

Follows The Elm Architecture (TEA):

```
┌──────────────┐         ┌──────────────┐         ┌──────────────┐
│   Terminal   │──Event─▶│    Event     │──Action─▶│    Update    │
│   (Input)    │         │   Handler    │         │   (state)    │
└──────────────┘         └──────────────┘         └──────┬───────┘
                                                         │
┌──────────────┐         ┌──────────────┐         ┌──────▼───────┐
│   Terminal   │◀─Render─│     View     │◀─State──│    Model     │
│   (Output)   │         │   (draw)     │         │   (state)    │
└──────────────┘         └──────────────┘         └──────────────┘
```

## Key Files

| File | Purpose |
|------|---------|
| `src/main.rs` | Entry point, terminal setup, event loop |
| `src/app/state.rs` | App state (Model) - holds all data |
| `src/app/actions.rs` | Action enum - all possible state changes |
| `src/ui/draw.rs` | View function - renders the UI |
| `src/event/handler.rs` | Key mapping - converts keys to actions |
| `src/kubernetes/client.rs` | K8s client using kube-rs |
| `src/kubernetes/reconcile.rs` | Flux CLI wrapper |

## Common Operations

### Build

```bash
cargo build
cargo build --release
```

### Run

```bash
cargo run
RUST_LOG=debug cargo run
```

### Test

```bash
cargo test
```

### Format & Lint

```bash
cargo fmt
cargo clippy
```

## Key Dependencies

- `ratatui 0.29` - TUI framework
- `crossterm 0.28` - Terminal backend
- `tokio 1.43` - Async runtime
- `kube 0.99` - Kubernetes client
- `k8s-openapi 0.24` - K8s API types
- `color-eyre 0.6` - Error handling
- `serde 1.0` - Serialization

## Kubernetes Integration

- **Listing**: Uses kube-rs for type-safe, fast, native async listing
- **Reconciling**: Uses flux CLI to handle dependency resolution and source refresh

```rust
// Listing (kube-rs)
let kustomizations = api.list(&ListParams::default()).await?;

// Reconciling (flux CLI)
flux reconcile kustomization <name> -n <namespace> [--with-source]
```

## Adding New Features

1. Add new action in `src/app/actions.rs`
2. Handle action in `src/app/state.rs` update() method
3. Map key to action in `src/event/handler.rs`
4. Update UI in `src/ui/draw.rs` or relevant widget

## Theme Customization

Colors are defined in `src/ui/theme.rs` using Tailwind CSS-inspired palette.
