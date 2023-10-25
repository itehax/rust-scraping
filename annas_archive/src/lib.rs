use bypass_cloudflare::bypass_cloudflare;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::Duration;
use thirtyfour::{By, WebDriver, WebElement};
use undetected_chromedriver::chrome;
use url::Url;

pub async fn scrape_annas_archive(book: &str) -> Result<(), Box<dyn Error>> {
    let driver = chrome().await?;
    driver.maximize_window().await?;

    let url = generate_url(book)?;

    driver.goto(url).await?;
    thread::sleep(Duration::from_secs(1));

    scrape_all(&driver).await?;

    Ok(())
}

fn generate_url(book: &str) -> Result<Url, Box<dyn Error>> {
    let mut url = Url::parse("https://annas-archive.org/search")?;
    url.set_query(Some(&format!("q={}", book)));
    Ok(url)
}

async fn scrape_all(driver: &WebDriver) -> Result<Vec<BookInfo>, Box<dyn Error>> {
    driver
        .execute("window.scrollTo(0, document.body.scrollHeight);", vec![])
        .await?;
    thread::sleep(Duration::from_secs(1));

    let books = get_books(driver).await?;

    let books_info = get_books_info(books).await?;

    let choosen_book = books_info.get(0).expect("Unable to get the book");

    download_book(choosen_book, driver).await?;
    Ok(books_info)
}

async fn get_books(driver: &WebDriver) -> Result<Vec<WebElement>, Box<dyn Error>> {
    let books = driver
        .find_all(By::Css(
            r"body > main > form > div.flex.w-\[100\%\] > div.min-w-\[0\] > div > div.h-\[125\] > a",
        ))
        .await?;
    Ok(books)
}

async fn get_books_info(books: Vec<WebElement>) -> Result<Vec<BookInfo>, Box<dyn Error>> {
    let mut books_info = vec![];
    for book in &books {
        let name = get_book_name(book).await?;

        let info = get_book_info(book).await?;
        let parts: Vec<&str> = info.split(',').collect();

        let _language = parts
            .first()
            .unwrap_or(&" ")
            .trim()
            .split('[')
            .next()
            .unwrap()
            .trim()
            .to_string();

        let extension = parts.get(1).unwrap_or(&" ").trim().to_string();

        let _size_str = parts.get(2).unwrap_or(&" ").trim().to_string();

        let mut filename = parts.get(3).unwrap_or(&" ").trim().to_string();

        if let Some(last_part) = parts.get(4) {
            let missing_part = format!(",{}", last_part.trim());
            filename.push_str(&missing_part);
        }

        let download_link = get_book_link(book).await?;
        books_info.push(BookInfo::new(name, extension, download_link));
    }
    Ok(books_info)
}

async fn get_book_name(book: &WebElement) -> Result<String, Box<dyn Error>> {
    let book_name = book
        .find(By::Css(
            r" div.relative.top-\[-1\].pl-4.grow.overflow-hidden > h3",
        ))
        .await?
        .text()
        .await?;
    Ok(book_name)
}

async fn get_book_info(book: &WebElement) -> Result<String, Box<dyn Error>> {
    let info = book.find(By::Css(r" div.relative.top-\[-1\].pl-4.grow.overflow-hidden > div.truncate.text-xs.text-gray-500")).await?.text().await?;
    Ok(info)
}

async fn download_book(book_info: &BookInfo, driver: &WebDriver) -> Result<(), Box<dyn Error>> {
    let md5_link = get_md5_link(driver, book_info).await?;
    bypass_cloudflare(driver, md5_link).await?;
    thread::sleep(Duration::from_secs(2));

    download_file(driver, book_info).await?;
    Ok(())
}

async fn get_md5_link(driver: &WebDriver, book_info: &BookInfo) -> Result<Url, Box<dyn Error>> {
    driver.goto(book_info.download_link.clone()).await?;
    let cloudflare_link = driver
        .find(By::Css(
            "#md5-panel-downloads > div:nth-child(2) > ul > li:nth-child(1) > a.js-download-link",
        ))
        .await?
        .attr("href")
        .await?
        .expect("Unable to get download link");
    let md5_link = append_to_base_url(cloudflare_link)?;
    Ok(md5_link)
}

async fn download_file(driver: &WebDriver, book_info: &BookInfo) -> Result<(), Box<dyn Error>> {
    let first_window = driver
        .windows()
        .await?
        .last()
        .expect("Unable to get last windows")
        .clone();
    driver.switch_to_window(first_window).await?;
    let download_link = driver
        .find(By::Css("body > main > p:nth-child(2) > a"))
        .await?
        .attr("href")
        .await?
        .expect("Unable to get book donwload link");
    driver.clone().quit().await?;

    show_progress_and_download(download_link, book_info).await?;

    Ok(())
}

async fn show_progress_and_download(
    download_link: String,
    book_info: &BookInfo,
) -> Result<(), Box<dyn Error>> {
    let body = reqwest::get(&download_link).await?;

    let total_size = body.content_length().ok_or(format!(
        "Failed to get content length from '{}'",
        &download_link
    ))?;

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{msg} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})",
            )?
            .progress_chars("#>-"),
    );
    pb.set_message(format!("Downloading {}", book_info.name));

    let path = format!("{}.{}", book_info.name, book_info.extension);
    let mut file = File::create(&path)?;
    let mut downloaded: u64 = 0;

    let mut stream = body.bytes_stream();
    while let Some(item) = stream.next().await {
        let chunk = item.or(Err("Error while downloading file".to_string()))?;
        file.write_all(&chunk)
            .or(Err("Error while writing to file".to_string()))?;

        downloaded += chunk.len() as u64;
        pb.set_position(downloaded);
    }

    pb.finish_with_message("Done!");
    Ok(())
}

async fn get_book_link(book: &WebElement) -> Result<Url, Box<dyn Error>> {
    let path = book.attr("href").await?.expect("Unable to get book link");
    let url = append_to_base_url(path)?;
    Ok(url)
}

fn append_to_base_url(path: String) -> Result<Url, Box<dyn Error>> {
    let mut url = Url::parse("https://annas-archive.org/")?;
    url.set_path(&path);
    Ok(url)
}

#[derive(Clone)]
pub struct BookInfo {
    name: String,
    extension: String,
    download_link: Url,
}

impl BookInfo {
    fn new(name: String, extension: String, download_link: Url) -> Self {
        Self {
            name,
            extension,
            download_link,
        }
    }
}
