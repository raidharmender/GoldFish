## GoldFish (Rust)

Rust/Tokio + `actix-web` service scaffold inspired by `GoldFish_design.md`.

### What’s included

- **Public API** (default `:4000`): `GET /api/v1/health`
- **OpenAPI spec**: `GET /api/spec`
- **Swagger UI**: `GET /swaggerui/`
- **Metrics endpoint** (default `:4001`): `GET /metrics`

### Quick start

```bash
cargo run -p goldfish-api
```

### Dev dependencies (optional)

```bash
docker compose up -d
```
