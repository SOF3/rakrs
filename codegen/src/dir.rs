// rakrs
// Copyright (C) SOFe
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Error, Result};

// I am not making this more generic or separating it because of the specific use case this applies
// to.
pub fn imp() -> Result<TokenStream> {
    let file = proc_macro::Span::call_site().source_file();
    if !file.is_real() {
        return Err(Error::new(
            Span::call_site(),
            "dirmod!() must not be called from a macro.",
        ));
    }

    let dir = file.path().parent().unwrap().read_dir().map_err(map_err)?;
    let mut out = vec![];
    for peer in dir {
        let peer = peer.map_err(map_err)?;
        let path = peer.path();
        if path.is_file() && path.extension().filter(|ext| *ext == "rs").is_some() {
            let name = path.file_name().expect("Path should end with .rs");
            if name != "mod.rs" {
                let ident = match name.to_str() {
                    Some(ident) => ident,
                    None => {
                        return Err(Error::new(
                            Span::call_site(),
                            "File name {} is not valid UTF-8",
                        ))
                    }
                };
                let ident = Ident::new(&ident[0..ident.len() - 3], Span::call_site());
                out.push(quote!(mod #ident;));
            }
        }
    }
    Ok(quote! {
        #(#out)*
    })
}

fn map_err(err: std::io::Error) -> Error {
    Error::new(Span::call_site(), err)
}
