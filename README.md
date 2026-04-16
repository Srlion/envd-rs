# envd

A simple, fast `.env` loader for Rust — runtime loading and compile-time macros.

## Installation

```toml
[dependencies]
envd = "0.1.0"
```

## Runtime loading

Load `.env` into the process environment:

```rust
fn main() {
    envd::load().expect("failed to load .env");

    let db_url = std::env::var("DATABASE_URL").unwrap();
}
```

Existing variables are preserved by default. To override:

```rust
envd::load_with("./.env", envd::Options::new().override_existing()).unwrap();
```

Or parse into a `HashMap` without touching the environment:

```rust
let map = envd::parse(include_str!(".env"));
```

## Compile-time macros

Embed values from `.env` directly into your binary. Missing keys become **compile errors** — no surprises at runtime.

```rust
const API_KEY: &str = envd::var!("API_KEY");
const PORT: u16 = envd::var!("PORT": u16);
```

Or fall back to the compile-time value when the env var isn't set at runtime:

```rust
let port: u16 = envd::dyn_var!("PORT": u16);
```

Supported types: `str`, `u8`–`u64`, `usize`, `i8`–`i64`, `isize`, `f32`, `f64`, `bool`.

## .env Syntax

```sh
# Comments are supported
APP_NAME=my_app
PORT=8080

# Quoted values
GREETING="Hello, World!"

# Single quotes — no variable expansion
LITERAL='$NOT_A_VAR'

# Variable expansion
BASE_URL=https://example.com
API_URL=${BASE_URL}/api

# Default value if VAR is unset or empty
TIMEOUT=${CUSTOM_TIMEOUT:-30}

# export prefix is supported
export SECRET_KEY=supersecret
```

## License

MIT
