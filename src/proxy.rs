use reqwest::{Client, Response};
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let PROXY_ENDPOINT: String = std::env::var("PROXY_ENDPOINT").expect("PROXY_ENDPOINT must be set.");
    let PROXY_TEST_TARGET: String = std::env::var("PROXY_TEST_TARGET").expect("PROXY_TEST_TARGET must be set.");
    let proxy = reqwest::Proxy::https(PROXY_ENDPOINT).unwrap();
    let clientProxy = Client::builder().proxy(proxy).build().unwrap();
    let response = clientProxy.get(PROXY_TEST_TARGET).send().await.unwrap();
    println!("{:?}", response.text().await.unwrap());
}