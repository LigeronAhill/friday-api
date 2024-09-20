mod currency;
use crate::Result;
const DATABASE: &str = "friday";
#[derive(Clone)]
pub struct Storage {
    database: mongodb::Database,
}
impl Storage {
    pub async fn new(uri: &str) -> Result<Self> {
        let client = mongodb::Client::with_uri_str(uri).await?;
        let database = client.database(DATABASE);
        Ok(Self { database })
    }
}
