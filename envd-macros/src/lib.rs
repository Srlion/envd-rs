use std::{collections::HashMap, fs, sync::OnceLock};

use proc_macro::{Literal, TokenStream, TokenTree};
use proc_macro2::Span;
use syn::{Error, LitStr, parse_macro_input};

static ENV_VARS: OnceLock<HashMap<String, String>> = OnceLock::new();
static ENV_PATH: OnceLock<String> = OnceLock::new();

fn init(span: Span, p: &str) -> Result<&'static HashMap<String, String>, Error> {
    ENV_PATH.get_or_init(|| p.to_string());
    if let Some(vars) = ENV_VARS.get() {
        return Ok(vars);
    }
    let contents = fs::read_to_string(p)
        .map_err(|e| Error::new(span, format!("failed to read {p:?}: {e}")))?;
    let _ = ENV_VARS.set(envd_parser::parse(&contents));
    Ok(ENV_VARS.get().expect("just set"))
}

#[proc_macro]
pub fn var(input: TokenStream) -> TokenStream {
    let lit_str = parse_macro_input!(input as LitStr);
    let span = lit_str.span();
    let path = ENV_PATH.get().map(|s| s.as_str()).unwrap_or("./.env");
    match init(span, path) {
        Err(e) => e.to_compile_error().into(),
        Ok(vars) => match vars.get(&lit_str.value()) {
            Some(v) => TokenTree::Literal(Literal::string(v)).into(),
            None => Error::new(
                span,
                format!("env var `{}` not found in `{path}`", lit_str.value()),
            )
            .to_compile_error()
            .into(),
        },
    }
}

#[proc_macro]
pub fn set_path(input: TokenStream) -> TokenStream {
    let lit_str = parse_macro_input!(input as LitStr);
    let span = lit_str.span();
    match init(span, &lit_str.value()) {
        Ok(_) => TokenStream::new(),
        Err(e) => e.to_compile_error().into(),
    }
}
