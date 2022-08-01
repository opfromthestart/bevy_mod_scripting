#![allow(dead_code, unused_variables, unused_features)]

pub(crate) mod common;
pub(crate) mod lua;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    braced, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token::{Brace, Bracket, Paren},
    ItemFn, Result, Token, Type,
};

pub(crate) use {common::*, lua::*};

#[derive(Default, Debug, Clone)]
struct EmptyToken;

impl Parse for EmptyToken {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self)
    }
}
impl ToTokens for EmptyToken {
    fn to_tokens(&self, tokens: &mut TokenStream2) {}
}

struct NewtypeList {
    paren: Paren,
    module_headers: TokenStream2,
    sq_bracket1: Bracket,
    additional_types: Punctuated<Type, Token![,]>,
    sq_bracket2: Bracket,
    new_types: Punctuated<Newtype, Token![,]>,
}

impl Parse for NewtypeList {
    fn parse(input: ParseStream) -> Result<Self> {
        let h;
        let f;
        let g;
        Ok(Self {
            paren: parenthesized!(h in input),
            module_headers: h.parse()?,
            sq_bracket1: bracketed!(f in input),
            additional_types: f.parse_terminated(Type::parse)?,
            sq_bracket2: bracketed!(g in input),
            new_types: g.parse_terminated(Newtype::parse)?,
        })
    }
}

impl ToTokens for NewtypeList {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let module_headers = &self.module_headers;
        let external_types = &self.additional_types;
        let types = &self.new_types;
        tokens.extend(quote! {
            (#module_headers)
            [#external_types]
            [#types]
        })
    }
}

struct AdditionalImplBlock {
    impl_token: Token![impl],
    fn_token: Token![fn],
    impl_braces: Brace,
    functions: Punctuated<ItemFn, Token![;]>,
}

impl Parse for AdditionalImplBlock {
    fn parse(input: ParseStream) -> Result<Self> {
        let f;
        Ok(Self {
            impl_token: input.parse()?,
            fn_token: input.parse()?,
            impl_braces: braced!(f in input),
            functions: f.parse_terminated(ItemFn::parse)?,
        })
    }
}

/// A convenience macro which derives a lotta things to make your type work in all supported/enabled scripting languages.
/// 
/// This macro is used extensively in `bevy_mod_scripting/src/generated.rs`, for extensive usage examples see those macro invocations.
/// 
/// Right now the macro supports:
/// - primitive types surrounded in `Raw()`
///   - usize
///   - isize
///   - f32
///   - f64
///   - u128
///   - u64
///   - u32
///   - u16
///   - u8
///   - i128
///   - i64
///   - i32
///   - i16
///   - i8
///   - String
///   - bool
/// - other wrapper types generated by this macro surrounded in `Wrapper()`
/// - Both mutable and immutable references to any of the above (apart from on fields)
/// - the self type and receiver (self, &self or &mut self), if used in method must be followed by `:` to differentiate it from other self arguments  
/// Currently more complex types like: Option<T> and LuaWrapper<T> are not yet supported.
///  
/// # Example
/// ```rust
/// 
/// pub struct MyStruct{
///     my_field: bool
/// }
/// 
/// impl MyStruct {
///     pub fn do_something(&self) -> bool {
///         self.my_field
///     }
/// }
/// impl_script_newtype!(
///     MyStruct:
///       Fields(
///         my_field: Raw(bool)
///       ) + Methods(
///         do_something(&self:) -> Raw(bool)
///       ) 
/// )
/// ```
#[proc_macro]
pub fn impl_script_newtype(input: TokenStream) -> TokenStream {
    let new_type = parse_macro_input!(input as Newtype);

    let mut lua = LuaImplementor::default();

    match lua.generate(&new_type) {
        Ok(v) => v.into(),
        Err(e) => e.into_compile_error().into(),
    }
}
