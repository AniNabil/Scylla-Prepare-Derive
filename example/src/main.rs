use scylla_prepare_derive::PrepareScylla;
use scylla::prepared_statement::PreparedStatement;
use scylla::Session;
use scylla::transport::errors::QueryError;


#[derive(PrepareScylla)]
pub struct PreparedStatements {
    get_user: PreparedStatement,
    get_group: PreparedStatement,
    //...
}

#[tokio::main]
async fn main() {

}