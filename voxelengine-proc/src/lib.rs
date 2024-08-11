extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(nested))]
struct MetaDataStructAttributes {
    nested: String,
}

#[proc_macro_derive(Fields, attributes(nested))]
pub fn process_fields_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree

    let item: proc_macro2::TokenStream = input.into();

    let mut ast: DeriveInput = syn::parse2(item).unwrap();

    let struct_: syn::DataStruct = match ast.data {
        syn::Data::Struct(data) => data,
        _ => panic!("Usage of #[Modbus] on a non-struct type"),
    };
    for field in struct_.fields.iter(){
        for attribute in field.attrs.iter(){
            if attribute.path.is_ident("nested"){
                
            }
        }
    }
    TokenStream::new()
}

fn impl_trait_a(name: &syn::Ident, fields: Fields) -> proc_macro2::TokenStream {
    quote! {
        impl Indexable for #name {
            fn nfields() -> usize {
                fields.len();
            }
        }
    }
}

fn process_fields(fields: &Fields) -> proc_macro2::TokenStream {
    let mut field_processors = Vec::new();

    for field in fields {
        if let Some(ident) = &field.ident {
            let field_name = ident.to_string();
            let field_type = &field.ty;

            let field_processor = match field_type {
                Type::Path(type_path) => {
                    let path = &type_path.path;
                    if let Some(last_segment) = path.segments.last() {
                        // Check if the type is a struct (not a basic type)
                        if last_segment.ident != "String" && last_segment.ident != "i32" {
                            let nested_fields = quote! {
                                println!("Field: {}, Type: {:?}", #field_name, stringify!(#path));
                                // Recursively process nested fields
                                // This requires more sophisticated parsing and handling
                            };
                            nested_fields
                        } else {
                            quote! {
                                println!("Field: {}, Type: {:?}", #field_name, stringify!(#path));
                            }
                        }
                    } else {
                        quote! {
                            println!("Field: {}, Type: {:?}", #field_name, "Unknown");
                        }
                    }
                }
                _ => quote! {
                    println!("Field: {}, Type: {:?}", ident, "Unknown");
                },
            };

            field_processors.push(field_processor);
        }
    }

    quote! {
        #(#field_processors)*
    }
}
