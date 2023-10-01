use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    airbnb::scrape_airbnb("Rome").await?;
    Ok(())
}
