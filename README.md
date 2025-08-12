# Kromer API

The `kromer-api` crate provides a strongly typed interface for interacting with the [Kromer2](https://github.com/ReconnectedCC/kromer2) server. It omits backwards compatability parts of the API, such as those that deal with Krist mining.

This crate isn't on crates.io and I don't plan on publishing it, so to begin using it add the following to your `Cargo.toml`:

```toml
[dependencies]
kromer-api = { git = "https://github.com/Laincy/kromer-api" }
```

Everything should be pretty well documented. For more info checkout the docs locally by running the following:

```bash
cargo doc --open
```

## Features

This crate provides a the full suite of Kromer2 HTTP endpoints. Websocket support is planned, and lookup endpoints will be implemented once Kromer2 supports is.
