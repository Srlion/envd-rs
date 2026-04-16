use std::str::FromStr;

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    Error, Ident, LitStr, Token,
    parse::{Parse, ParseStream},
};

pub struct VarInput {
    pub key: LitStr,
    pub ty: Option<Ident>,
}

impl Parse for VarInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key = input.parse()?;
        let ty = input
            .parse::<Option<Token![:]>>()?
            .map(|_| input.parse())
            .transpose()?;
        Ok(Self { key, ty })
    }
}

fn parse_as<T: FromStr + ToTokens>(v: &str, span: Span) -> Result<TokenStream, Error>
where
    T::Err: std::fmt::Display,
{
    v.parse::<T>()
        .map(|x| quote!(#x))
        .map_err(|e| Error::new(span, format!("failed to parse `{v}`: {e}")))
}

pub fn typed(value: &str, ty: Option<&Ident>, span: Span) -> Result<TokenStream, Error> {
    match ty.map(|t| t.to_string()).as_deref() {
        None | Some("str") => Ok(quote!(#value)),
        Some("u8") => parse_as::<u8>(value, span),
        Some("u16") => parse_as::<u16>(value, span),
        Some("u32") => parse_as::<u32>(value, span),
        Some("u64") => parse_as::<u64>(value, span),
        Some("usize") => parse_as::<usize>(value, span),
        Some("i8") => parse_as::<i8>(value, span),
        Some("i16") => parse_as::<i16>(value, span),
        Some("i32") => parse_as::<i32>(value, span),
        Some("i64") => parse_as::<i64>(value, span),
        Some("isize") => parse_as::<isize>(value, span),
        Some("f32") => parse_as::<f32>(value, span),
        Some("f64") => parse_as::<f64>(value, span),
        Some("bool") => parse_as::<bool>(value, span),
        Some(other) => Err(Error::new(span, format!("unsupported type `{other}`"))),
    }
}
