# Pali Server

[![CI](https://github.com/pali-org/server/actions/workflows/ci.yml/badge.svg)](https://github.com/pali-org/server/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Cloudflare Workers backend for the Pali todo management ecosystem.

## Usage

```bash
# Setup
wrangler d1 create pali-database
wrangler d1 migrations apply pali-database
wrangler deploy

# Initialize server (creates first admin key)
curl -X POST https://your-worker.workers.dev/initialize
```

## API

```bash
# Create todo
curl -H "X-API-Key: your-key" \
  -d '{"title":"Buy groceries"}' \
  https://your-worker.workers.dev/todos
```

See [API.md](API.md) for complete endpoint documentation.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.