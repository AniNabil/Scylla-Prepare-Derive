#![recursion_limit = "256"]
extern crate proc_macro;

//use scylla::{prepared_statement::PreparedStatement, serialize::value, transport::errors::QueryError, Session};
use proc_macro::TokenStream;
use quote::{quote, TokenStreamExt};
use syn::{Data, DeriveInput, Fields, Ident};

#[proc_macro_derive(PrepareScylla)]
pub fn prepare_scylla_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name: Ident = ast.ident.clone();
    let fields: Vec<Ident> = match_fields(ast);
    println!("{:?}", name);
    println!("{:?}", fields);

    // Build the trait implementation
    write_code(name, fields)
}

fn match_fields(input: DeriveInput) -> Vec<Ident> {
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

fn write_code(name: Ident, field_names: Vec<Ident>) -> TokenStream {
    let fields_init: proc_macro2::TokenStream = field_names.iter().map(|x| {
        quote! {
            #x: #x(session).await?,
        }
    }).collect();
    let fields_functions: proc_macro2::TokenStream = field_names.iter().map(|x| {
        let ident_string = x.to_string();
        let mut string: String = "../../../cql/queries/".to_string();
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
