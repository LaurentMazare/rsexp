// This deriver is used to convert between some struct/enum types and the Sexp type.
// It might be more efficient to write a direct serialization/deserialization deriver,
// directly or via serde.
//
// TODO: support sexp.option, default values etc.
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_quote, DataEnum, DataUnion, DeriveInput, FieldsNamed, FieldsUnnamed, GenericParam,
};

#[proc_macro_derive(SexpOf)]
pub fn sexp_of_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_sexp_of(&ast)
}

fn impl_sexp_of(ast: &DeriveInput) -> TokenStream {
    let DeriveInput {
        ident,
        data,
        generics,
        ..
    } = ast;
    let mut generics = generics.clone();
    for param in &mut generics.params {
        if let GenericParam::Type(type_param) = param {
            type_param.bounds.push(parse_quote!(SexpOf))
        }
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let impl_fn = match data {
        syn::Data::Struct(s) => match &s.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => {
                let fields = named.iter().map(|field| {
                    let name = field.ident.as_ref().unwrap();
                    let name_str = name.to_string();
                    quote! { rsexp::list(&[rsexp::atom(#name_str.as_bytes()), self.#name.sexp_of()]) }
                });
                quote! {rsexp::list(&[#(#fields),*])}
            }
            syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let num_fields = unnamed.len();
                let fields = (0..num_fields).map(|index| {
                    let index = syn::Index::from(index);
                    quote! { self.#index.sexp_of() }
                });
                quote! {rsexp::list(&[#(#fields),*])}
            }
            syn::Fields::Unit => {
                unimplemented!()
            }
        },
        syn::Data::Enum(DataEnum { variants, .. }) => {
            let cases = variants.iter().map(|variant| {
                let variant_ident = &variant.ident;
                let variant_str = variant_ident.to_string();
                let cstor = quote! { rsexp::atom(#variant_str.as_bytes()) };
                let (pattern, sexp) = match &variant.fields {
                    syn::Fields::Named(FieldsNamed { named, .. }) => {
                        let args = named.iter().map(|field| field.ident.as_ref().unwrap());
                        let fields = named.iter().map(|field| {
                            let name = field.ident.as_ref().unwrap();
                            let name_str = name.to_string();
                            quote! { rsexp::list(&[rsexp::atom(#name_str.as_bytes()), #name.sexp_of()]) }
                        });
                        let sexp =
                            if variant.fields.is_empty() {
                                quote! { #cstor }
                            } else {
                                quote! { rsexp::list(&[#cstor, #(#fields),*]) }
                            };
                        (quote! { { #(#args),* } }, sexp)
                    }
                    syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                        let num_fields = unnamed.len();
                        let args = (0..num_fields).map(|index| format_ident!("arg{}", index));
                        let fields = args.clone().map(|arg| quote! { #arg.sexp_of() });
                        let sexp =
                            if num_fields == 0 {
                                quote! { #cstor }
                            } else {
                                quote! { rsexp::list(&[#cstor, #(#fields),*]) }
                            };
                        (quote! { (#(#args),*) }, sexp)
                    }
                    syn::Fields::Unit => (quote! {}, quote! { #cstor }),
                };
                quote! {
                    #ident::#variant_ident #pattern => { #sexp }
                }
            });
            quote! {
                match self {
                    #(#cases)*
                }
            }
        }
        syn::Data::Union(DataUnion { union_token, .. }) => {
            return syn::Error::new_spanned(&union_token, "union is not supported")
                .to_compile_error()
                .into();
        }
    };

    let output = quote! {
        impl #impl_generics rsexp::SexpOf for #ident #ty_generics #where_clause {
            fn sexp_of(&self) -> rsexp::Sexp {
                #impl_fn
            }
        }
    };

    output.into()
}

#[proc_macro_derive(OfSexp)]
pub fn of_sexp_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_of_sexp(&ast)
}

fn impl_of_sexp(ast: &DeriveInput) -> TokenStream {
    let DeriveInput {
        ident,
        data,
        generics,
        ..
    } = ast;
    let ident_str = ident.to_string();
    let mut generics = generics.clone();
    for param in &mut generics.params {
        if let GenericParam::Type(type_param) = param {
            type_param.bounds.push(parse_quote!(OfSexp))
        }
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let of_sexp_fn = match data {
        syn::Data::Struct(s) => match &s.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => {
                let fields = named.iter().map(|field| field.ident.as_ref().unwrap());
                let mk_fields = named.iter().map(|field| {
                    let name = field.ident.as_ref().unwrap();
                    let name_str = name.to_string();
                    quote! {
                        let #name = match __map.remove(#name_str) {
                            Some(sexp) => sexp,
                            None => return Err(rsexp::IntoSexpError::MissingFieldsInStruct {
                                type_: #ident_str,
                                field: #name_str,
                            })
                        };
                    }
                });
                quote! {
                    let mut __map: std::collections::HashMap<String, Sexp> = __s.of_sexp();
                    #(#mk_fields)*
                    if !__map.is_empty() {
                        let fields = __map.into_keys().collect();
                        return Err(rsexp::IntoSexpError::ExtraFieldsInStruct {
                            type_: #ident_str,
                            fields,
                        })
                    }
                    Ok(#ident { #(#fields),* })
                }
            }
            syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let num_fields = unnamed.len();
                let fields = (0..num_fields).map(|index| format_ident!("__field{}", index));
                let fields_ = fields.clone();
                let fields_list = quote! { #(rsexp::OfSexp::of_sexp(#fields)?),*};
                quote! {
                    match __s.extract_list(#ident_str)? {
                        [#(#fields_,)*] => Ok(#ident(#fields_list)),
                        l => Err(rsexp::IntoSexpError::ListLengthMismatch {
                            type_: #ident_str,
                            expected_len: #num_fields,
                            list_len: l.len(),
                        }),
                    }
                }
            }
            syn::Fields::Unit => unimplemented!(),
        },
        syn::Data::Enum(DataEnum { variants, .. }) => {
            let cases = variants.iter().enumerate().map(|(variant_index, variant)| {
                let variant_ident = &variant.ident;
                let (mk_fields, fields) = match &variant.fields {
                    syn::Fields::Named(FieldsNamed { named, .. }) => {
                        let fields = named.iter().map(|field| field.ident.as_ref().unwrap());
                        let mk_fields = named.iter().map(|field| {
                            let name = field.ident.as_ref().unwrap();
                            quote! { let #name = of_sexp()?; }
                        });
                        (quote! { #(#mk_fields)* }, quote! { { #(#fields),* } })
                    }
                    syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                        let num_fields = unnamed.len();
                        let fields = (0..num_fields).map(|index| format_ident!("__field{}", index));
                        let mk_fields = (0..num_fields).map(|index| {
                            let ident = format_ident!("__field{}", index);
                            quote! { let #ident = of_sexp()?; }
                        });
                        (quote! { #(#mk_fields)* }, quote! { (#(#fields),*) })
                    }
                    syn::Fields::Unit => (quote! {}, quote! {}),
                };
                quote! {
                    #variant_index => {
                        #mk_fields
                        Ok(#ident::#variant_ident #fields)
                    }
                }
            });
            quote! {
                match __s {
                    #(#cases)*
                    rsexp::Sexp::Atom(atom) =>
                        Err(rsexp::IntoSexpError::UnknownConstructorForEnum {
                            type_: #ident_str,
                            constructor: String::from_utf8_lossy(atom),
                        }),
                    rsexp::Sexp::List(l) if l.is_empty() => Err(rsexp::IntoSexpError::NotAConstructorForEnum { type_: #ident_str })
                    rsexp::Sexp::List(l) => {
                        match l[0] {
                            rsexp::Sexp::Atom(atom) =>
                                Err(rsexp::IntoSexpError::UnknownConstructorForEnum {
                                    type_: #ident_str,
                                    constructor: String::from_utf8_lossy(atom),
                                }),
                            _ => Err(rsexp::IntoSexpError::NotAConstructorForEnum { type_: #ident_str })
                        }
                    }
                }
            }
        }
        syn::Data::Union(DataUnion { union_token, .. }) => {
            return syn::Error::new_spanned(&union_token, "union is not supported")
                .to_compile_error()
                .into();
        }
    };

    let output = quote! {
        impl #impl_generics rsexp::OfSexp for #ident #ty_generics #where_clause {
            fn of_sexp(__s: &rsexp::Sexp) -> std::result::Result<Self, rsexp::IntoSexpError> {
                #of_sexp_fn
            }
        }
    };

    output.into()
}
