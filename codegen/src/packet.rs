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

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Attribute, Data, DeriveInput, Error, Fields, Ident, Result, Type};

pub fn imp(item: DeriveInput) -> Result<TokenStream> {
    let item_name = &item.ident;

    let (writer, reader) = match &item.data {
        Data::Struct(data) => {
            let writer = write_fields(
                &data.fields,
                |ident| quote!(self.#ident),
                |i, _| quote!(self.#i),
            )?;
            let reads = read_fields(&data.fields)?;

            (writer, quote!(Self #reads))
        }
        Data::Enum(data) => {
            let endian = match find_attr(&item.attrs, "little_endian") {
                Some(_) => quote!(LittleEndian),
                None => quote!(BigEndian),
            };

            let repr_attr = find_attr(&item.attrs, "repr")
                .ok_or_else(|| Error::new(item.span(), "Enum packets must declare #[repr]"))?;
            let repr_ty = repr_attr.parse_args::<Ident>()?;
            let (repr_write, repr_read) = match repr_ty.to_string().as_str() {
                "u8" => (quote!(write_u8), quote!(read_u8)),
                "u16" => (
                    quote!(write_u16::<::byteorder::#endian>),
                    quote!(read_u16::<::byteorder::#endian>),
                ),
                "u32" => (
                    quote!(write_u32::<::byteorder::#endian>),
                    quote!(read_u32::<::byteorder::#endian>),
                ),
                "u64" => (
                    quote!(write_u64::<::byteorder::#endian>),
                    quote!(read_u64::<::byteorder::#endian>),
                ),
                _ => Err(Error::new(
                    repr_attr.tokens.span(),
                    "Only repr(u[8|16|32|64]) enums are supported",
                ))?,
            };

            let mut write_vars = Vec::with_capacity(data.variants.len());
            let mut read_vars = Vec::with_capacity(data.variants.len());
            for variant in &data.variants {
                let var_name = &variant.ident;
                let (_, discrim) = variant.discriminant.as_ref().ok_or_else(|| {
                    Error::new(
                        variant.span(),
                        "All enum packet variants must have discriminants",
                    )
                })?;
                let fields_pat = pat_fields(&variant.fields);
                let fields_write = write_fields(
                    &variant.fields,
                    |ident| {
                        let ident = Ident::new(&format!("variant_{}", ident), ident.span());
                        quote!(#ident)
                    },
                    |id, span| {
                        let ident = generate_ident(id, span);
                        quote!(#ident)
                    },
                )?;
                let fields_read = read_fields(&variant.fields)?;

                write_vars.push(quote!(#item_name::#var_name #fields_pat => {
                    w.#repr_write(#discrim)?;
                    #fields_write
                }));
                read_vars.push(quote!(#discrim => #item_name::#var_name #fields_read));
            }

            let writer = quote! {
                match self {
                    #(#write_vars),*
                }
            };
            let reader = quote! {
                let id = r.#repr_read()?;
                match id {
                    #(#read_vars,)*
                    _ => Err(::std::io::Error::new(::std::io::ErrorKind::Other, format!("Unexpected enum variant {:?}", r)))?,
                }
            };

            (writer, reader)
        }
        _ => Err(Error::new(
            item.span(),
            "Unions cannot be derived as Packet",
        ))?,
    };

    Ok(quote! {
        #[automatically_derived]
        impl crate::protocol::CanIo for #item_name {
            fn write<W: ::std::io::Write>(&self, mut w: W) -> ::std::io::Result<()> {
                #writer
                Ok(())
            }

            fn read<R: ::std::io::Read>(mut r: R) -> ::std::io::Result<Self> {
                Ok(#reader)
            }
        }
    })
}

fn find_attr<'a, I, S>(attr: I, name: S) -> Option<&'a Attribute>
where
    I: IntoIterator<Item = &'a Attribute>,
    S: AsRef<str>,
{
    attr.into_iter()
        .filter(|attr| attr.path.is_ident(&name))
        .next()
}

fn write_fields<F, G>(fields: &Fields, access_named: F, access_unnamed: G) -> Result<TokenStream>
where
    F: Fn(&Ident) -> TokenStream,
    G: Fn(usize, Span) -> TokenStream,
{
    let ret = match fields {
        Fields::Named(fields) => {
            let fields: Vec<TokenStream> = fields
                .named
                .iter()
                .map(|field| {
                    let ident = field.ident.as_ref().unwrap();
                    let accessor = access_named(ident);
                    write_field(&field.ty, &accessor)
                })
                .collect::<Result<_>>()?;
            quote!(#(#fields)*)
        }
        Fields::Unnamed(fields) => {
            let fields: Vec<TokenStream> = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, field)| {
                    let accessor = access_unnamed(i, field.span());
                    write_field(&field.ty, &accessor)
                })
                .collect::<Result<_>>()?;
            quote!(#(#fields)*)
        }
        Fields::Unit => quote!(),
    };
    Ok(ret)
}

fn read_fields(fields: &Fields) -> Result<TokenStream> {
    let ret = match fields {
        Fields::Named(fields) => {
            let fields: Vec<TokenStream> = fields
                .named
                .iter()
                .map(|field| {
                    let field_name = &field.ident;
                    let read_expr = read_field(&field.ty)?;
                    Ok(quote!(#field_name: #read_expr))
                })
                .collect::<Result<_>>()?;
            quote!({ #(#fields),* })
        }
        Fields::Unnamed(fields) => {
            let fields: Vec<TokenStream> = fields
                .unnamed
                .iter()
                .map(|field| {
                    let read_expr = read_field(&field.ty)?;
                    Ok(quote!(#read_expr))
                })
                .collect::<Result<_>>()?;
            quote!(( #(#fields),* ))
        }
        Fields::Unit => quote!(),
    };
    Ok(ret)
}

fn pat_fields(fields: &Fields) -> TokenStream {
    match fields {
        Fields::Named(fields) => {
            let fields = fields
                .named
                .iter()
                .map(|field| field.ident.as_ref().unwrap());
            quote!({ #(#fields),* })
        }
        Fields::Unnamed(fields) => {
            let fields = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, field)| generate_ident(i, field.span()));
            quote!(( #(#fields),* ))
        }
        Fields::Unit => quote!(),
    }
}

fn write_field(_ty: &Type, expr: &TokenStream) -> Result<TokenStream> {
    Ok(quote! {
        crate::protocol::CanIo::write(&(#expr), &mut w)?;
    })
}

fn read_field(ty: &Type) -> Result<TokenStream> {
    Ok(quote! {
        {
            let var: #ty = crate::protocol::CanIo::read(&mut r)?;
            var
        }
    })
}

fn generate_ident(i: usize, span: Span) -> Ident {
    Ident::new(&format!("generated_ident_{}", i), span)
}
