use basic_text_internals::{is_basic_text, is_basic_text_substr};
use proc_macro::TokenStream;
use quote::quote_spanned;
use syn::{parse_macro_input, LitStr};

/// `TextStr` literal support: `text!("string literal")`.
///
/// Returns a `'static &TextStr` containing the provided string literal.
#[proc_macro]
pub fn text(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let span = input.span();

    if !is_basic_text(&input.value()) {
        return (quote_spanned! { span =>
            compile_error!("string literal is not Basic Text")
        })
        .into();
    }

    (quote_spanned! { span =>
        unsafe { ::basic_text::TextStr::from_text_unchecked(#input) }
    })
    .into()
}

/// `TextSubstr` literal support: `text_substr!("string literal\u{200e}")`.
///
/// Returns a `'static &TextSubstr` containing the provided string literal.
#[proc_macro]
pub fn text_substr(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let span = input.span();

    if !is_basic_text_substr(&input.value()) {
        return (quote_spanned! { span =>
            compile_error!("string literal is not Basic Text substring")
        })
        .into();
    }

    (quote_spanned! { span =>
        unsafe { ::basic_text::TextSubstr::from_text_unchecked(#input) }
    })
    .into()
}
