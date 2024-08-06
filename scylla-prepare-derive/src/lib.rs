#![recursion_limit = "256"]
extern crate proc_macro;

use scylla::{prepared_statement::PreparedStatement, transport::errors::QueryError, Session};
use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(PrepareScylla)]
pub fn prepare_scylla_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_scylla_derive(&ast)
}

fn impl_scylla_derive(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl PreparedStatements {
            pub async fn new(session: &Session) -> Result<Self, QueryError> {
                Ok(
                    PreparedStatements{
                    get_user: get_user(session).await?,
                    get_group: get_group(session).await?
                    }
                )
            }
        }


        async fn get_user(session: &Session) -> Result<PreparedStatement, QueryError> {
            let stmt = include_str!("../../../cql/queries/get_user.cql");
            session.prepare(stmt).await
        }

        async fn get_group(session: &Session) -> Result<PreparedStatement, QueryError> {
            let stmt = include_str!("../../../cql/queries/get_group.cql");
            session.prepare(stmt).await
        }
    };
    gen.into()
}