use serde::Serialize;
use std::error::Error;
use std::thread;
use std::time::Duration;
use thirtyfour::{
    prelude::{ElementWaitable, WebDriverError},
    By, DesiredCapabilities, WebDriver, WebElement,
};
use url::Url;

pub async fn scrape_airbnb(place: &str) -> Result<(), Box<dyn Error>> {
    let driver = initialize_driver().await?;
    let url = Url::parse("https://www.airbnb.it/")?;

    driver.goto(url).await?;
    thread::sleep(Duration::from_secs(2));

    search_location(&driver, place).await?;
    thread::sleep(Duration::from_secs(2));

    scrape_all(driver).await?;

    Ok(())
}

async fn scrape_all(driver: WebDriver) -> Result<(), Box<dyn Error>> {
    driver
        .execute("window.scrollTo(0, document.body.scrollHeight);", vec![])
        .await?;
    thread::sleep(Duration::from_secs(1));

    let mut wtr = csv::Writer::from_path("airbnb.csv")?;

    loop {
        if let Ok(next_page_button) = driver.find(By::Css("#site-content > div > div.p1szzjq8.dir.dir-ltr > div > div > div > nav > div > a.l1ovpqvx.c1ytbx3a.dir.dir-ltr")).await {

            match next_page_button.is_clickable().await? {
                true => {

                    //start extracting data

                    let house_elems = get_house_elements(&driver).await?;

                    for house_elem in house_elems {

                        let bnb_details = BnbDetails::from(house_elem).await?;

                        wtr.serialize(bnb_details)?;

                    }
                    load_next_page(next_page_button, &driver).await?;
                }
                false => {
                    break
                },
            }
        } else {
            let house_elems = get_house_elements(&driver).await?;

            for house_elem in house_elems {

                let bnb_details = BnbDetails::from(house_elem).await?;
                wtr.serialize(bnb_details)?;
            }
            break;
        }
    }
    Ok(())
}

async fn load_next_page(
    next_page_button: WebElement,
    driver: &WebDriver,
) -> Result<(), Box<dyn Error>> {
    next_page_button.click().await?;
    thread::sleep(Duration::from_secs(2));

    driver
        .execute("window.scrollTo(0, document.body.scrollHeight);", vec![])
        .await?;
    thread::sleep(Duration::from_secs(1));

    Ok(())
}

async fn get_house_elements(driver: &WebDriver) -> Result<Vec<WebElement>, WebDriverError> {
    driver.find_all(By::Css("#site-content > div > div:nth-child(2) > div > div > div > div > div.gsgwcjk.g8ge8f1.g14v8520.dir.dir-ltr > div > div > div.c1l1h97y.dir.dir-ltr > div > div > div > div > div > div.g1qv1ctd.c1v0rf5q.dir.dir-ltr")).await
}

async fn initialize_driver() -> Result<WebDriver, WebDriverError> {
    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:9515", caps).await?;
    driver.maximize_window().await?;
    Ok(driver)
}

async fn search_location(driver: &WebDriver, place: &str) -> Result<(), WebDriverError> {
    click_choose_place(driver).await?;

    write_place(driver, place).await?;

    click_search_button(driver).await?;

    Ok(())
}


async fn click_choose_place(driver: &WebDriver) -> Result<(), WebDriverError> {
    driver
        .find(By::Css("body > div:nth-child(8) > div > div > div:nth-child(1) > div > div.cd56ld.cb80sj1.dir.dir-ltr > div.h1ta6hky.dir.dir-ltr > div > div > div > header > div > div.c1ujpdn9.dir.dir-ltr > div.l1sjr04j.l1x4ovsg.llb1jct.lc9d3st.dir.dir-ltr > div > span.ieg7dag.dir.dir-ltr > button:nth-child(1)"))
        .await?.click().await?; 

    Ok(())
}

async fn write_place(driver: &WebDriver, place: &str) -> Result<(), WebDriverError> {
    let input = driver
        .find(By::Css("#bigsearch-query-location-input"))
        .await?;
    input.wait_until().clickable().await?;

    input.send_keys(place).await?;

    Ok(())
}

async fn click_search_button(driver: &WebDriver) -> Result<(), WebDriverError> {
    driver.find(By::Css("#search-tabpanel > div.ir2ixub.dir.dir-ltr > div.c111bvlt.c1gh7ier.dir.dir-ltr > div.c1ddhymz.cggll98.dir.dir-ltr > div.s1t4vwjw.dir.dir-ltr > button")).await?.click().await?;
    Ok(())
}

#[derive(Debug, Serialize)]
struct BnbDetails {
    title: String,
    description: String,
    host: String,
    availability: String,
    price: String,
    star: String,
}

impl BnbDetails {
    async fn from(house_elem: WebElement) -> Result<Self, WebDriverError> {
        let title = BnbDetails::get_title(&house_elem).await?;
        let description = BnbDetails::get_description(&house_elem).await?;
        let host = BnbDetails::get_host(&house_elem).await?;
        let availability = BnbDetails::get_availability(&house_elem).await?;
        let price = BnbDetails::get_price(&house_elem).await?;
        let star = BnbDetails::get_star(&house_elem).await?;

        Ok(Self {
            title,
            description,
            host,
            availability,
            price,
            star,
        })
    }
    async fn get_title(house_elem: &WebElement) -> Result<String, WebDriverError> {
        house_elem
            .find(By::Css("div:nth-child(1)"))
            .await?
            .text()
            .await
    }
    async fn get_description(house_elem: &WebElement) -> Result<String, WebDriverError> {
        house_elem
            .find(By::Css("div:nth-child(2) > span"))
            .await?
            .text()
            .await
    }
    async fn get_host(house_elem: &WebElement) -> Result<String, WebDriverError> {
        let host = house_elem
            .find(By::Css("div:nth-child(3) > span > span"))
            .await;
        if let Ok(host) = host {
            host.text().await
        } else {
            house_elem
                .find(By::Css("div:nth-child(3) > span"))
                .await?
                .text()
                .await
        }
    }
    async fn get_availability(house_elem: &WebElement) -> Result<String, WebDriverError> {
        house_elem
            .find(By::Css("div:nth-child(4) > span > span"))
            .await?
            .text()
            .await
    }
    async fn get_price(house_elem: &WebElement) -> Result<String, WebDriverError> {
        house_elem
            .find(By::XPath("div[5]/div/div/span[1]/div/span[1]"))
            .await?
            .text()
            .await
    }

    async fn get_star(house_elem: &WebElement) -> Result<String, WebDriverError> {
        if let Ok(star) = house_elem
            .find(By::Css("span > span.r1dxllyb.dir.dir-ltr"))
            .await
        {
            return star.text().await;
        }
        Ok("No ratings available".into())
    }
}
