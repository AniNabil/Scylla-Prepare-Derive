#![recursion_limit = "256"]
extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields, Ident, Lit, Meta, parse_macro_input};

#[proc_macro_derive(PrepareScylla)]
pub fn prepare_scylla_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = parse_macro_input!(input as DeriveInput);
    // Initialize a variable to store the path to your queries
    let mut path = String::new();

    let name: Ident = ast.ident.clone();
    let fields: Vec<Ident> = match_fields(&ast);
    println!("{:?}", name);
    println!("{:?}", fields);

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

fn match_fields(input: &DeriveInput) -> Vec<Ident> {
    match &input.data {
        Data::Struct(value) => {
            match &value.fields {
                Fields::Named(value) => {
                    let mut ident_vec: Vec<Ident> = Vec::new();
                    let _ = &value.named.iter().for_each(|x|
                        ident_vec.push(x.ident.clone().expect("Unnamed Field"))
                    );
                    return ident_vec;
                },
                _ => {},
            } 
        },
        _ => {},
    }
    vec![]
}

fn write_code(name: Ident, field_names: Vec<Ident>, path: String) -> TokenStream {
    let fields_init: proc_macro2::TokenStream = field_names.iter().map(|x| {
        quote! {
            #x: #x(session).await?,
        }
    }).collect();
    let fields_functions: proc_macro2::TokenStream = field_names.iter().map(|x| {
        let ident_string = x.to_string();
        let mut string: String = path.to_string();
        string.push_str(&ident_string);
        string.push_str(".cql");
        quote! {
            async fn #x(session: &Session) -> Result<PreparedStatement, QueryError> {
            let stmt = include_str!(#string);
            session.prepare(stmt).await
            }

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