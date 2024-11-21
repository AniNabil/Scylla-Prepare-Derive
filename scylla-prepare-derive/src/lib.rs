#![recursion_limit = "256"]

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

    // Iterate over the attributes of the struct to find the `path` attribute
    for attr in &ast.attrs {
        if attr.path().is_ident("path") {
            match parse_path_attr(attr.clone()) {
                Ok(p) => path = p,
                Err(e) => panic!("Error parsing path attribute: {}", e),
            }
        }
    }

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
                        if value.ident.to_string() == "PreparedStatement" {
                            let mut file : String = "/".to_string();
                            file.push_str(path.as_str());
                            file.push_str(&ident_string);
                            file.push_str(".cql");
                            quote! {
                                #[allow(non_snake_case)]
                                async fn #x(session: &Session) -> Result<PreparedStatement, QueryError> {
                                    let stmt = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), #file));
                                    session.prepare(stmt).await
                                }
                                
                            }
                        } else if value.ident.to_string() == "Batch" {
                            let mut directory: String = "$CARGO_MANIFEST_DIR/".to_string();
                            directory.push_str(path.as_str());
                            directory.push_str(&ident_string);
                            quote! {
                                #[allow(non_snake_case)]
                                async fn #x(session: &Session) -> Result<Batch, QueryError> {
                                    let mut batch: Batch = Default::default();
                                    let statements_dir = include_dir!(#directory);
                                    for i in 0..statements_dir.files().count(){
                                        let mut file_name = i.to_string();
                                        file_name.push_str(".cql");
                                        let statement = statements_dir.get_file(file_name).unwrap().contents_utf8().unwrap();
                                        batch.append_statement(statement);
                                    }
                                    session.prepare_batch(&batch).await
                                }
                                
                            }
                        } else if value.ident.to_string() == "Vec" {
                            match &value.arguments {
                                syn::PathArguments::AngleBracketed(angle_bracketed_generic_arguments) => {
                                    match angle_bracketed_generic_arguments.args.first() {
                                        Some(first_argument) => {
                                            match first_argument {
                                                syn::GenericArgument::Type(found_type) => {
                                                    match found_type {
                                                        Type::Path(type_path) => {
                                                            match type_path.path.segments.first() {
                                                                Some(inner_type) => {
                                                                    if inner_type.ident.to_string() == "PreparedStatement" {
                                                                        let mut directory: String = "$CARGO_MANIFEST_DIR/".to_string();
                                                                        directory.push_str(path.as_str());
                                                                        directory.push_str(&ident_string);
                                                                        quote! {
                                                                            #[allow(non_snake_case)]
                                                                            async fn #x(session: &Session) -> Result<Vec<PreparedStatement>, QueryError> {
                                                                                let mut statements: Vec<PreparedStatement> = vec![];
                                                                                let statements_dir = include_dir!(#directory);
                                                                                for i in 0..statements_dir.files().count(){
                                                                                    let mut file_name = i.to_string();
                                                                                    file_name.push_str(".cql");
                                                                                    let statement = statements_dir.get_file(file_name).unwrap().contents_utf8().unwrap();
                                                                                    statements.push(session.prepare(statement).await?);
                                                                                }
                                                                                return Ok(statements);
                                                                            }
                                                                        }
                                                                    } else {
                                                                        quote! {}
                                                                    }
                                                                }
                                                                _ => {
                                                                    quote! {}
                                                                }
                                                            }
                                                        }
                                                        _ => {
                                                            quote! {}
                                                        }
                                                    }
                                                }
                                                _ => quote! {}
                                            }
                                        }
                                        _ => {
                                            quote! {}
                                        }
                                    }
                                },
                                _ => {
                                    quote! {}
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
        #[allow(non_snake_case)]
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