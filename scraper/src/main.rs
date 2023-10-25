use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // airbnb::scrape_airbnb("Rome").await?;

    // annas_archive::scrape_annas_archive("questa è l'acqua").await?;

    // let driver = undetected_chromedriver::chrome().await?;
    // let link = url::Url::parse("https://nowsecure.nl")?;
    // bypass_cloudflare::bypass_cloudflare(&driver, link).await?;

    Ok(())
}
