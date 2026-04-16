use std::{collections::HashMap, fs, sync::Mutex, time::SystemTime};

use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{Error, parse_macro_input};

mod parse;
use parse::VarInput;

const ENV_PATH: &str = "./.env";

struct Cache {
    mtime: SystemTime,
    vars: HashMap<String, String>,
}

static CACHE: Mutex<Option<Cache>> = Mutex::new(None);

fn lookup(key: &str, span: Span) -> Result<Option<String>, Error> {
    let meta = fs::metadata(ENV_PATH)
        .map_err(|e| Error::new(span, format!("failed to read {ENV_PATH:?}: {e}")))?;
    let mtime = meta
        .modified()
        .map_err(|e| Error::new(span, format!("failed to read mtime: {e}")))?;

    let mut cache = CACHE.lock().unwrap();
    if cache.as_ref().map(|c| c.mtime) != Some(mtime) {
        let contents = fs::read_to_string(ENV_PATH)
            .map_err(|e| Error::new(span, format!("failed to read {ENV_PATH:?}: {e}")))?;
        *cache = Some(Cache {
            mtime,
            vars: envd_parser::parse(&contents),
        });
    }
    Ok(cache.as_ref().unwrap().vars.get(key).cloned())
}

/// Embeds an env var from `.env` at compile time.
///
/// Reads `./.env` during compilation and inlines the value as a literal.
/// Fails to compile if the key is missing or the value can't parse as the
/// requested type.
///
/// # Syntax
/// ```ignore
/// var!("KEY")          // &'static str
/// var!("KEY": u16)     // parsed at compile time
/// ```
///
/// Supported types: `str`, `u8`–`u64`, `usize`, `i8`–`i64`, `isize`,
/// `f32`, `f64`, `bool`.
#[proc_macro]
pub fn var(input: TokenStream) -> TokenStream {
    let VarInput { key, ty } = parse_macro_input!(input as VarInput);
    let span = key.span();

    let value = match lookup(&key.value(), span) {
        Ok(Some(v)) => v,
        Ok(None) => {
            return Error::new(
                span,
                format!("env var `{}` not found in `{ENV_PATH}`", key.value()),
            )
            .to_compile_error()
            .into();
        }
        Err(e) => return e.to_compile_error().into(),
    };

    match parse::typed(&value, ty.as_ref(), span) {
        Ok(t) => t.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// Reads an env var at runtime, falling back to `.env` at compile time.
///
/// Uses `std::env::var` if set, otherwise the value baked in from `./.env`.
/// Panics if the env var is set but malformed. Fails to compile if the key
/// is missing from `.env` or the fallback can't parse as the requested type.
///
/// # Syntax
/// ```ignore
/// dyn_var!("KEY")          // String
/// dyn_var!("KEY": u16)     // parsed at runtime
/// ```
///
/// Supported types: `str`, `u8`–`u64`, `usize`, `i8`–`i64`, `isize`,
/// `f32`, `f64`, `bool`.
#[proc_macro]
pub fn dyn_var(input: TokenStream) -> TokenStream {
    let VarInput { key, ty } = parse_macro_input!(input as VarInput);
    let span = key.span();
    let key_str = key.value();

    let fallback_value = match lookup(&key_str, span) {
        Ok(Some(v)) => v,
        Ok(None) => {
            return Error::new(
                span,
                format!("env var `{}` not found in `{ENV_PATH}`", key_str),
            )
            .to_compile_error()
            .into();
        }
        Err(e) => return e.to_compile_error().into(),
    };

    // Validate the fallback parses at compile time. We discard the tokens
    // and re-emit inside the runtime expression below so the fallback is
    // evaluated in the caller's context with the correct type.
    if let Err(e) = parse::typed(&fallback_value, ty.as_ref(), span) {
        return e.to_compile_error().into();
    }

    parse::runtime_or(&key_str, ty.as_ref(), &fallback_value, span)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
