# Bera-Reth ExEx Template

Template for building indexers, trading bots, real-time analytics and more on Berachain using [Execution Extensions (ExEx)](https://reth.rs/exex/overview/).


## Quick Start

```bash
# 1. Clone this template (or your version) and beacon-kit
git clone https://github.com/berachain/bera-reth-exex-template.git
cd bera-reth-exex-template
git clone https://github.com/berachain/beacon-kit.git ../beacon-kit

# 2. Run with BeaconKit integration
make start-local  # Runs indefinitely, Ctrl+C to stop
```

This starts both BeaconKit and Bera-Reth with the ExEx, logging PoL transactions in real-time.

## Production Deployment

For mainnet and testnet deployments, build the production Docker image:

```bash
make docker-build
```

The resulting image can be used as a **drop-in replacement** for the official bera-reth Docker image, as long as the bera-reth version in `Cargo.toml` matches the required version for your network. Simply replace `berachain/bera-reth` with your custom ExEx image in your deployment configuration.

## Using This Template

When using this repository as a template, update the following:

### Required Changes
1. **Package name** in `Cargo.toml`:
   ```toml
   [package]
   name = "your-exex-name"  # Change from "bera-reth-exex-template"
   ```

2. **Docker labels** in `Dockerfile`:
   ```dockerfile
   LABEL org.opencontainers.image.source=https://github.com/your-org/your-repo
   ```


## Development

Edit `src/exex.rs` to customize the exex. Find more information on the [official docs](https://reth.rs/exex/overview/#how-do-i-build-an-execution-extension)


Run `make help` to see all available commands.

## License

Apache-2.0