# envd

A simple, fast `.env` file loader for Rust — with both runtime loading and compile-time macro support.

## Installation

```toml
[dependencies]
envd = "0.1.0"
```

## Usage

### Runtime loading

Load a `.env` file into the process environment:

```rust
fn main() {
    envd::load().expect("failed to load .env");

    let db_url = std::env::var("DATABASE_URL").unwrap();
}
```

By default, existing environment variables are **not** overridden. To override them:

```rust
envd::load_with("./.env", envd::Options::new().override_existing()).unwrap();
```

You can also parse a `.env` file directly into a `HashMap` without touching the environment:

```rust
let map = envd::parse(include_str!(".env"));
```

### Compile-time macros

Embed env values directly into your binary at compile time.

```rust
const API_KEY: &str = envd::var!("API_KEY");
const PORT: &str = envd::var!("PORT");

fn main() {
    let port = envd::var!("PORT");
    println!("Listening on port {PORT}");
}
```

```rust
// any other file in your crate
const DB_URL: &str = envd::var!("DATABASE_URL");
```

If the variable is missing, you'll get a **compile error** — no surprises at runtime.

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
