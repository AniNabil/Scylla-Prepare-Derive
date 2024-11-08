#![recursion_limit = "256"]
extern crate proc_macro;
use std::fs;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Fields, Ident, Lit, Meta, Type};

#[derive(Debug)]
struct StructField {
    name: Ident,
    ty: Type
}

#[proc_macro_derive(PrepareScylla)]
pub fn prepare_scylla_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = parse_macro_input!(input as DeriveInput);
    // Initialize a variable to store the path to your queries
    let mut path = String::new();

    let name: Ident = ast.ident.clone();
    let fields: Vec<StructField> = match_fields(&ast);
    //println!("{:?}", name);
    //println!("{:?}", fields);

    // Iterate over the attributes of the struct to find the `path` attribute
    for attr in &ast.attrs {
        if attr.path().is_ident("path") {
            match parse_path_attr(attr.clone()) {
                Ok(p) => path = p,
                Err(e) => panic!("Error parsing path attribute: {}", e),
            }
        }
    }
    println!("{:?}", path);

    // Build the trait implementation
    write_code(name, fields, path)
}

fn match_fields(input: &DeriveInput) -> Vec<StructField> {
    match &input.data {
        Data::Struct(value) => {
            match &value.fields {
                Fields::Named(value) => {
                    let mut fields_vec: Vec<StructField> = Vec::new();
                    let _ = &value.named.iter().for_each(|x|{
                        fields_vec.push(
                            StructField { 
                                name: x.ident.clone().expect("Unnamed Field"),
                                ty: x.ty.clone()
                            }
                        );
                    }
                    );
                    return fields_vec;
                },
                _ => {},
            } 
        },
        _ => {},
    }
    vec![]
}

fn write_code(name: Ident, field_names: Vec<StructField>, path: String) -> TokenStream {
    let fields_init: proc_macro2::TokenStream = field_names.iter().map(|field| {
        let x = field.name.clone();
        quote! {
            #x: #x(session).await?,
        }
    }).collect();
    let fields_functions: proc_macro2::TokenStream = field_names.iter().map(|field| {
        match &field.ty {
            Type::Path(value) => {
                match value.path.segments.first() {
                    Some(value) => {
                        let x = field.name.clone();
                        let ident_string = x.to_string();
                        let mut string: String = path.to_string();
                        string.push_str(&ident_string);
                        if value.ident.to_string() == "PreparedStatement" {
                            string.push_str(".cql");
                            quote! {
                                async fn #x(session: &Session) -> Result<PreparedStatement, QueryError> {
                                    let stmt = include_str!(#string);
                                    session.prepare(stmt).await
                                }
                                
                            }
                        } else if value.ident.to_string() == "Batch" {
                            string.push_str("/");
                            let mut combined_statements = "".to_string();
                            let files = fs::read_dir(string.clone()).unwrap();
                            for i in 0..files.count() {
                                let mut file_path = string.to_string();
                                file_path.push_str(&i.to_string());
                                file_path.push_str(".cql");
                                println!("{}", file_path);
                                let split = fs::read_to_string(file_path);
                                match split {
                                    Ok(value) => {
                                        combined_statements.push_str("¦");
                                        combined_statements.push_str(&value);
                                    },
                                    Err(err) => {}
                                }
                            }
                            quote! {
                                async fn #x(session: &Session) -> Result<Batch, QueryError> {
                                    let files = fs::read_dir(#string).unwrap();
                                    let mut batch: Batch = Default::default();
                                    let combined_statements = include_str!(#combined_statements)
                                    let split_statements = combined_statements.split("¦")
                                    for i in 0..split_statements.len() {
                                        let mut file_path = #string.to_string();
                                        file_path.push_str(split_statements[i]);
                                        file_path.push_str(".cql");
                                        let stmt = fs::read_to_string(file_path);
                                        match stmt {
                                            Ok(value) => {
                                                batch.append_statement(value.as_str());
                                            },
                                            Err(err) => {}
                                        }
                                    }
                                    session.prepare_batch(&batch).await
                                }
                                
                            }
                        } else {
                            quote! {}
                        }
                    }
                    _ => quote! {}
                }
            }
            _ => quote! {},
        }
    }).collect();
    let gen = quote! {
        impl #name {
            pub async fn new(session: &Session) -> Result<Self, QueryError> {
                Ok(
                    #name{
                        #fields_init
                        }
                    )
                }
                }
        
                #fields_functions
    };
    gen.into()
}

// Function to parse the `path` attribute
fn parse_path_attr(attr: Attribute) -> Result<String, String> {
    let meta = attr.meta;
    if let Meta::NameValue(meta_name_value) = meta {
        if let syn::Expr::Lit(expr_lit) = meta_name_value.value {
            if let Lit::Str(lit_str) = &expr_lit.lit {
                return Ok(lit_str.value());
            }
        }
    }
    Err("Attribute format is incorrect".to_string())
}