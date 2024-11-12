use scylla::batch::Batch;
use scylla::transport::errors::QueryError;
use scylla::Session;
use scylla_prepare_derive::PrepareScylla;
use scylla::prepared_statement::PreparedStatement;
use include_dir::include_dir;


#[derive(PrepareScylla)]
#[path = "cql/queries/"]
pub struct PreparedStatements {
    get_user: PreparedStatement,
    get_group: PreparedStatement,
    use_code: Batch
    //...
}

#[tokio::main]
async fn main() {
}