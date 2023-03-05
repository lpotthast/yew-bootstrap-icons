#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use darling::*;
use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(yew_bootstrap_icons), supports(struct_any))]
struct MyInputReceiver {
    mod_name: Ident,

    json_path: String,

    prefix: String,

    always_add_prefix: bool,

    default: String,
}

#[proc_macro_derive(YewBootstrapIcons, attributes(yew_bootstrap_icons))]
#[proc_macro_error]
pub fn store(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let input: MyInputReceiver = match FromDeriveInput::from_derive_input(&ast) {
        Ok(args) => args,
        Err(err) => return darling::Error::write_errors(err).into(),
    };

    let mod_ident = input.mod_name;

    let json_keys = extract_keys(input.json_path);

    let prefix = Some(input.prefix.as_ref());
    let always_add_prefix = input.always_add_prefix;

    let key_with_type_ident = json_keys
        .iter()
        .map(|json_key| {
            let type_name = to_type_name(json_key.as_str(), prefix, always_add_prefix);
            let type_ident = Ident::new(type_name.as_str(), Span::call_site());
            (json_key, type_ident)
        })
        .collect::<Vec<_>>();

    let variants = key_with_type_ident
        .iter()
        .map(|(_, ident)| quote! { #ident })
        .collect::<Vec<_>>();

    // Example:  Bi::Bi1Circle => "1-circle",
    let to_json_key_match_arms = key_with_type_ident
        .iter()
        .map(|(key, ident)| quote! { Self::#ident => #key })
        .collect::<Vec<_>>();

    let default_variant = match key_with_type_ident
        .iter()
        .find(|(_, ident)| ident.to_string() == input.default)
    {
        Some((_key, ident)) => quote! { #ident },
        None => abort!(
            Span::call_site(),
            format!("Default '{}' is not a valid variant", input.default)
        ),
    };

    quote! {
        pub mod #mod_ident {
            #![forbid(unsafe_code)]
            #![deny(clippy::unwrap_used)]

            use std::fmt::Display;

            impl From<Bi> for yew::Classes {
                fn from(variant: Bi) -> Self {
                    yew::classes!(format!("bi-{}", variant.to_json_key()))
                }
            }

            #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, serde::Serialize, serde::Deserialize)]
            pub enum Bi {
                #(#variants),*
            }

            impl Bi {
                pub fn to_json_key(&self) -> &'static str {
                    match self {
                        #(#to_json_key_match_arms),*
                    }
                }
            }

            impl Default for Bi {
                fn default() -> Self {
                    Self::#default_variant
                }
            }

            impl std::fmt::Display for Bi {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", self.to_json_key())
                }
            }
        }
    }
    .into()
}

fn extract_keys(json_source: String) -> Vec<String> {
    let root = PathBuf::from("./");
    let path = PathBuf::from(json_source);
    let alt_path = PathBuf::from(format!("crates/yew-bootstrap-icons/{}", path.display()));

    let config = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(err) => match std::fs::read_to_string(&alt_path) {
            Ok(content) => content,
            Err(alt_err) => abort!(
                Span::call_site(),
                format!(
                    "Json source could not be read from '{path:?}' or '{alt_path:?}' in root {root:?}. err = {err}, alt_err = {alt_err}",
                    root = std::fs::canonicalize(&root)
                )
            ),
        },
    };

    let parsed: serde_json::Value = serde_json::from_str(&config).expect("Unable to parse JSON...");
    let keys: Vec<String> = parsed
        .as_object()
        .unwrap()
        .keys()
        .map(|k| k.clone())
        .collect();
    keys
}

// TODO: Use proc macro type name crate if possible.
fn to_type_name(string: &str, prefix: Option<&str>, always_add_prefix: bool) -> String {
    if let Some(some_prefix) = prefix {
        if let Some(first_prefix_char) = some_prefix.chars().next() {
            assert!(!first_prefix_char.is_numeric(), "asd");
        }
    }
    let mut out;
    let mut offset: i32;
    match always_add_prefix {
        true => {
            out = format!("{}{}", prefix.map_or("", |p| p), string);
            if let Some(some_prefix) = prefix {
                offset = some_prefix.chars().count() as i32;
            } else {
                offset = 0;
            }
        }
        false => {
            out = string.to_owned();
            offset = 0;
        }
    }
    // Setting this to true will uppercase the first possible character!
    let mut uppercase_next = true;
    for (i, c) in string.char_indices() {
        if !(c.is_ascii() && (c.is_alphanumeric() || c.is_numeric())) || c.is_whitespace() {
            out.remove((i as i32 + offset) as usize);
            offset -= 1;
            if c.is_ascii_punctuation() || c.is_whitespace() {
                uppercase_next = true;
            }
        } else {
            if uppercase_next {
                out.replace_range(
                    (i as i32 + offset) as usize..(i as i32 + offset + 1) as usize,
                    c.to_uppercase().to_string().as_str(),
                );
                uppercase_next = false;
            }
        }
    }

    let first_out_char_numeric;
    if let Some(first_out_char) = out.chars().next() {
        first_out_char_numeric = first_out_char.is_numeric();
    } else {
        first_out_char_numeric = false;
    }

    if !always_add_prefix && first_out_char_numeric {
        out = format!("{}{}", prefix.map_or("", |p| p), out);
    }

    if let Some(first_out_char) = out.chars().next() {
        assert!(!first_out_char.is_numeric());
    }
    out
}
