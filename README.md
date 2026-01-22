# flux-tui

[![CI](https://github.com/frank-bee/flux-tui/actions/workflows/ci.yml/badge.svg)](https://github.com/frank-bee/flux-tui/actions/workflows/ci.yml)

A minimalist, modern terminal UI for managing [Flux CD](https://fluxcd.io/) resources.

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ flux-tui                                         cluster: prod │ ns: flux-sys│
├──────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐                │
│  │ Kustomizations  │ │  HelmReleases   │ │   HelmCharts    │                │
│  └─────────────────┘ └─────────────────┘ └─────────────────┘                │
│ ┌────────────────────────────────────────────────────────────────────────┐  │
│ │ NAME                    │ READY │ STATUS              │ REVISION │ SUS │  │
│ ├─────────────────────────┼───────┼─────────────────────┼──────────┼─────┤  │
│ │►infrastructure          │  ✓    │ Applied revision... │ main@abc │  -  │  │
│ │ apps                    │  ✓    │ Applied revision... │ main@def │  -  │  │
│ │ cert-manager            │  ✗    │ Health check fail.. │ main@jkl │  -  │  │
│ │ ingress-nginx           │  ●    │ Reconciling...      │ main@mno │  -  │  │
│ └────────────────────────────────────────────────────────────────────────┘  │
├──────────────────────────────────────────────────────────────────────────────┤
│ ↑↓ Navigate │ Enter Details │ r Reconcile │ R +Source │ n Namespace │ q Quit│
└──────────────────────────────────────────────────────────────────────────────┘
```

## Features

- **View Flux resources**: Kustomizations, HelmReleases, and HelmCharts
- **Quick navigation**: Tab-based interface with vim-style keybindings
- **Reconcile resources**: Trigger reconciliation with or without source refresh
- **Suspend/Resume**: Toggle resource suspension
- **Namespace filtering**: Filter resources by namespace
- **Auto-refresh**: Automatically refreshes data every 5 seconds

## Installation

### From source

```bash
# Clone the repository
git clone https://github.com/frank-bee/flux-tui.git
cd flux-tui

# Build and install
cargo install --path .
```

### With Homebrew (coming soon)

```bash
brew install frank-bee/tap/flux-tui
```

## Prerequisites

- **kubectl**: Configured with access to a Kubernetes cluster
- **flux CLI**: Required for reconciliation operations
  ```bash
  # macOS
  brew install fluxcd/tap/flux

  # Linux
  curl -s https://fluxcd.io/install.sh | sudo bash
  ```

## Usage

```bash
# Use current kubectl context
flux-tui

# With specific kubeconfig
KUBECONFIG=/path/to/kubeconfig flux-tui
```

## Keybindings

| Key | Action |
|-----|--------|
| `↑` / `k` | Move selection up |
| `↓` / `j` | Move selection down |
| `←` / `h` | Previous tab |
| `→` / `l` | Next tab |
| `Tab` | Next tab |
| `Enter` | View resource details |
| `r` | Reconcile selected resource |
| `R` | Reconcile with source |
| `s` | Toggle suspend |
| `n` | Filter by namespace |
| `F5` | Refresh data |
| `q` / `Esc` | Quit |
| `g` | Go to top |
| `G` | Go to bottom |

## Status Icons

| Icon | Meaning |
|------|---------|
| ✓ | Ready |
| ✗ | Failed |
| ● | Reconciling |
| ⏸ | Suspended |

## Architecture

flux-tui is built with:

- **[ratatui](https://ratatui.rs/)**: Terminal UI framework
- **[kube-rs](https://kube.rs/)**: Kubernetes client for listing resources
- **[flux CLI](https://fluxcd.io/flux/cmd/)**: For reconciliation operations
- **[tokio](https://tokio.rs/)**: Async runtime

The application follows [The Elm Architecture](https://guide.elm-lang.org/architecture/) (TEA):

```
Event → Update (state) → View (render) → Event
```

## Development

```bash
# Run in development
cargo run

# Run with debug logging
RUST_LOG=debug cargo run

# Build release binary
cargo build --release
```

## License

MIT
