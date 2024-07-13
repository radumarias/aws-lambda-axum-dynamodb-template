# shuttle-axum-postgres

Migration from `lambda` to [shuttle.rs](https://shuttle.rs).

# Steps

Their docs for [migrating](https://docs.shuttle.rs/migration/migrating-to-shuttle).

You can see here the [steps](https://github.com/radumarias/aws-lambda-axum-dynamodb/commit/e15019dc21e348e4f4c662c270a44572168d6314) for this project.

# Run locally

```bash
cargo shuttle run
```

Open [http://127.0.0.1:8000/v1/results/550e8400-e29b-41d4-a716-446655440000?page=1&per_page=42](http://127.0.0.1:8000/v1/results/550e8400-e29b-41d4-a716-446655440000?page=1&per_page=42).

# Deploy

```bash
cargo shuttle deploy
```
