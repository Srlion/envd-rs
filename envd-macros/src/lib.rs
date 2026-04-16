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
