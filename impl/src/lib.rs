// MIT License
//
// Copyright (c) 2019 Lukas Lueg (lukas.lueg@gmail.com)
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! `restruct` is used to interpret binary data stored in files or other sources or convert
//! between C structs and Rust types and when using a parser-generator is considered
//! disproportionate.
//! It is a brainchild of [Python's struct-module](https://docs.python.org/3/library/struct.html).
//!
//! See the `restruct_derive`-crate for documentation.

#![recursion_limit = "256"]
#![feature(external_doc)]

#[doc(include = "../README.md")]
#[allow(dead_code)]
type _READMETEST = ();

use quote::ToTokens;

mod generator;
mod parser;

/// Types derived using this crate implement this trait. One can refer to the
/// types use for packing/unpacking using e.g.
/// `<Self as restruct::Struct>::Packed`
pub trait Struct {
    /// The type used for the packed form, a [u8; _]-array.
    type Packed;
    /// The type used for the unpacked form, a tuple.
    type Unpacked;
}

pub fn derive(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let derive = Derive::new(syn::parse2(input).unwrap());

    let format = parser::parse(&derive.format).expect("Failed to parse format-string");

    let comp = crate::generator::Compilation::new(derive.name.clone(), derive.generics, format);

    if derive.debug_output {
        eprintln!(
            "Token stream for `{}`, format \"{}\":\n {}",
            derive.name,
            &derive.format,
            &comp.to_string()
        );
    }
    comp.into_token_stream()
}

struct Derive {
    pub name: syn::Ident,
    pub generics: syn::Generics,
    pub format: String,
    pub debug_output: bool,
}

impl Derive {
    fn new(ast: syn::DeriveInput) -> Self {
        let name = ast.ident;
        let generics = ast.generics;

        let mut format = String::new();
        let mut debug_output = false;

        for attr in ast.attrs {
            match attr.interpret_meta() {
                Some(syn::Meta::NameValue(ref name_value)) if name_value.ident == "fmt" => {
                    match &name_value.lit {
                        syn::Lit::Str(string) => format.push_str(&string.value()),
                        _ => panic!("fmt attribute must be a string."),
                    }
                }
                Some(syn::Meta::NameValue(ref name_value))
                    if name_value.ident == "debug_output" =>
                {
                    match &name_value.lit {
                        syn::Lit::Bool(b) => debug_output = b.value,
                        _ => panic!("debug_output attribute must be a bool."),
                    }
                }
                Some(syn::Meta::Word(ref ident)) if ident == "debug_output" => debug_output = true,
                _ => {}
            }
        }

        Self {
            name,
            generics,
            format,
            debug_output,
        }
    }
}
