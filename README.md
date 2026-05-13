# goggin-rs-process-watch

`goggin-rs-process-watch` is a terminal application for configuring development
processes, workflows, and documentation shortcuts from a TOML file.

Current implemented behavior focuses on CLI parsing, config loading, and config
validation. Process supervision and the tui-realm terminal UI are planned future
work, but are not implemented in the current code.

## Usage

Run with the default config file in the current directory:

```sh
goggin-rs-process-watch run
```

Run with an explicit config file:

```sh
goggin-rs-process-watch run --config path/to/process-watch.toml
```

When `--config` is omitted, the application reads `process-watch.toml` from the
current directory. Watched paths are validated relative to the directory that
contains the config file.

## Config File

The config file is TOML. Unknown fields are rejected.

At least one service is required:

```toml
[services.api]
label = "API server"
command = ["cargo", "run", "-p", "api"]
watch = ["Cargo.toml", "crates/api"]
port = 3000
env = { RUST_LOG = "debug" }
```

Services describe long-running processes. Each service supports:

- `label`: Optional display label.
- `command`: Required command and arguments. It must include at least one
  non-empty argument.
- `watch`: Optional files or directories that must exist.
- `port`: Optional primary service port. If provided, it must be greater than
  `0`.
- `env`: Optional environment variables.
- `readiness`: Optional HTTP or TCP readiness check.
- `log_relay`: Optional log forwarding settings.

HTTP readiness checks require an `http://` or `https://` URL. If
`expected_status` is provided, it must be between 100 and 599:

```toml
[services.api.readiness]
kind = "http"
url = "http://127.0.0.1:3000/health"
expected_status = 200
```

TCP readiness checks require a non-empty host and a port greater than 0:

```toml
[services.database.readiness]
kind = "tcp"
host = "127.0.0.1"
port = 5432
```

Log relay targets may be omitted, but an enabled relay cannot use a blank target:

```toml
[services.api.log_relay]
enabled = true
target = "dev.api"
```

## Workflows

Workflows define one-shot commands such as checks, tests, or documentation
builds:

```toml
[workflows.check]
label = "Check workspace"
command = ["cargo", "check", "--workspace", "--all-targets"]
watch = ["Cargo.toml", "src"]
```

Workflow commands follow the same validation rules as service commands. Workflow
watch paths must also exist relative to the config file.

## Documentation Shortcuts

Documentation shortcuts point to local generated docs or served URLs:

```toml
[docs.rustdoc]
label = "Rustdoc"
path = "target/doc"
workflow = "docs"

[docs.frontend]
label = "Frontend preview"
url = "http://127.0.0.1:8080"
workflow = "frontend"
```

Each docs entry must define exactly one of `path` or `url`. If `workflow` is
provided, it must refer to a workflow defined in the same config.

See [`examples/api-web-common.toml`](examples/api-web-common.toml) for a fuller
sample with API, frontend, database, cache, workflow, and docs entries.

## Development

Run the test suite:

```sh
cargo test
```

Build project documentation:

```sh
cargo doc --no-deps
```
