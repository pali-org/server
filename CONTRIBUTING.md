# Contributing to Pali Server

## Development Setup

```bash
# Clone the repository
git clone https://github.com/pali-org/server.git
cd server

# Install wrangler CLI
npm install -g wrangler

# Setup local development environment
cp wrangler.toml.example wrangler.toml
# Edit wrangler.toml with your Cloudflare account details
```

## Before Submitting PRs

Run the local CI checks to ensure your code meets quality standards:

```bash
./scripts/ci-check.sh
```

This comprehensive script validates:
- **Formatting**: `cargo fmt --check`
- **Linting**: `cargo clippy` (native + WASM targets)
- **Compilation**: Native and WASM builds
- **Tests**: Full test suite
- **Binary Size**: Cloudflare Workers limits check
- **Security**: Dependency vulnerability audit

## Cloudflare Workers Specifics

- **WASM Target**: Always test with `wasm32-unknown-unknown`
- **Binary Size**: Keep under 5MB for Cloudflare limits
- **No Standard Library**: Limited to `#[no_std]` compatible crates
- **Testing**: Consider Cloudflare Workers runtime limitations

## Code Standards

- **Zero warnings**: All clippy warnings must be fixed
- **WASM compatible**: Code must compile for `wasm32-unknown-unknown`
- **Formatted code**: Use `cargo fmt` before committing
- **Security focused**: Handle API keys and authentication carefully

## Architecture

This serverless backend integrates with:
- **[Terminal Client](https://github.com/pali-org/terminal)** - CLI/TUI interfaces
- **[Types](https://github.com/pali-org/types)** - Shared type definitions
- **Cloudflare D1** - Serverless SQLite database
- **Cloudflare Workers** - Global edge runtime