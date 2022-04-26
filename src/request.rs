use reqwest;
use std::error::Error;

pub async fn post(body: String, url: &str) -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();
    let _response = client
        .post(url)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await?;
    Ok(())
}

