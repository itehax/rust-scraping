use std::error::Error;
use std::thread;
use std::time::Duration;
use thirtyfour::{prelude::ElementWaitable, By, WebDriver};
use url::Url;
pub async fn bypass_cloudflare(
    driver: &WebDriver,
    download_link: Url,
) -> Result<(), Box<dyn Error>> {
    driver
        .execute(
            &format!(r#"window.open("{}", "_blank");"#, download_link.as_str()),
            vec![],
        )
        .await?;
    thread::sleep(Duration::from_secs(3));
    let first_window = driver
        .windows()
        .await?
        .first()
        .expect("Unable to get first windows")
        .clone();
    driver.switch_to_window(first_window).await?;
    driver.close_window().await?;
    let first_window = driver
        .windows()
        .await?
        .last()
        .expect("Unable to get last windows")
        .clone();
    driver.switch_to_window(first_window).await?;

    driver.enter_frame(0).await?;

    let button = driver.find(By::Css("#challenge-stage")).await?;

    button.wait_until().clickable().await?;
    thread::sleep(Duration::from_secs(2));
    button.click().await?;
    Ok(())
}
