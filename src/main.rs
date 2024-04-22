use std::time::Duration;
use std::{fs::File, sync::Arc};
use std::io::Write;
use reqwest::{Client, Response};
use serde_json::Value;
use tokio::sync::{Mutex, Semaphore};
use dotenv::dotenv;



const BATCH_SIZE:i32 = 8;
const SLEEP_TIME:u64 = 1;
const FAILED_SLEEP_TIME:u64 = 5;
const END:i32 = 325647;
//const START:i32 = 321277;


#[tokio::main]
async fn main() {
    dotenv().ok();
    // List /saves folder and pick the last file to get the last game_id
    // Get the last game_id from the last file in /saves sorted by filename
    let files = std::fs::read_dir("./saves");
    let last_file = files.unwrap().reduce(|f, l| {
        let f = f.unwrap();
        let l = l.unwrap();
        if f.file_name() > l.file_name() {
            return Ok(f);
        } else {
            return Ok(l);
        }
    }).unwrap().unwrap();
    
    let last_file_name = last_file.file_name().into_string().unwrap();
    let last_game_id = last_file_name.split(".").collect::<Vec<&str>>()[0].parse::<i32>().unwrap();

    let mut curr_batch = last_game_id + 1;
    //print the last game_id
    println!("Last game_id: {}", last_game_id);

    let totalToDownload = END - last_game_id;
    println!("Downloading {} games", totalToDownload);

    let mut use_proxy = 0;
    // Stop if we reach the end
    while curr_batch < END {
        println!("Start of batch {} - ", curr_batch);
        // Estimate the time to download the rest of the games
        let timeLeft = (END - curr_batch) * SLEEP_TIME as i32;
        println!("Estimated time left: {} minutes", timeLeft / 60);
        
        let failed = get_batch(curr_batch, use_proxy).await;
        use_proxy = (use_proxy+1) % 3;
        if (failed.len() > 0) {
            println!("Failed URLs: {:?} - Retrying batch", failed);
            tokio::time::sleep(Duration::from_secs(FAILED_SLEEP_TIME)).await;

        } else {
            curr_batch += BATCH_SIZE;
        }
    
        // Print end of batch
        println!("End of batch {}", curr_batch);
    }
}



async fn get_batch(curr_batch: i32, use_proxy: i32) -> Vec<String> {

    let base_string = std::env::var("BASE").unwrap();
    let BASE:&str = base_string.as_str();

    let proxy_string = std::env::var("PROXY").unwrap();
    let PROXY:&str = proxy_string.as_str();

    let proxy2_string = std::env::var("PROXY2").unwrap();
    let PROXY2:&str = proxy2_string.as_str();

    // Build a vec of urls starting at curr_batch and ending at curr_batch + BATCH_SIZE made from get_url
    let mut urls: Vec<String> = Vec::new();
    for i in 0..BATCH_SIZE {
        urls.push(get_url(BASE,curr_batch + i));
    }

    let mut client = Client::new();
    // Switch use_proxy, 0 no proxy, 1 proxy 1, 2 proxy 2
    match use_proxy {
        1 => {
            println!("--- PROXY [1] ---");
            let proxy = reqwest::Proxy::https(PROXY).unwrap();
            client = Client::builder().proxy(proxy).build().unwrap();
        },
        2 => {
            println!("--- PROXY 2 ---");
            let proxy = reqwest::Proxy::https(PROXY2).unwrap();
            client = Client::builder().proxy(proxy).build().unwrap();
        }
        _ => println!("No proxy"),
    }

    let semaphore = Arc::new(Semaphore::new(BATCH_SIZE as usize));
    
    let mut handles = vec![];
    // let failed_urls: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let failed_urls = Arc::new(Mutex::new(Vec::new()));

    // For each url in urls, spawn a task in parralel that will get the url and decodeAndSave the response
    for url in urls {
        let cloned_client = client.clone();
        let _permit = semaphore.clone().acquire_owned().await.unwrap();
        let failed_urls_clone = failed_urls.clone();
        let task = tokio::spawn(async move {
            let current_url = url.clone();
            println!("{} asked ", current_url);
            let response = cloned_client.get(url).send().await;
            match response {
                Ok(response) => {
                    println!("{} received! [{:?}] - ({:?})", current_url, response.status(), response.content_length());
                    if (response.status() == 503) {
                        println!("{} 503 error - TOO FAST", current_url);
                        //
                        //             let mut failed = failed_urls_clone.lock().await; // Lock the mutex to access the data
                    //                  failed.push(current_url);
                        failed_urls_clone.lock_owned().await.push(current_url);
                    } 
                    else if (response.status() == 404){
                        println!("{} 404 error - outdated", current_url);
                    }
                    else {
                        decodeAndSave(response, current_url).await;
                    }   
                },
                Err(e) => {
                    println!("{} failed - {:?}", current_url, e);
                    failed_urls_clone.lock_owned().await.push(current_url);
                }
            }
        });
        handles.push(task);
        //tokio::task::block_in_place(|| task.await.unwrap());
    }

    for handle in handles {
        match handle.await {
            Ok(_) => {},
            Err(e) => println!("Thread failed: {:?}", e),
        }
    }

    let final_failed_urls = Arc::try_unwrap(failed_urls).expect("Arc still has multiple owners")
    .into_inner();
println!("Failed URLs: {:?}", final_failed_urls);

    tokio::time::sleep(Duration::from_secs(SLEEP_TIME)).await;
    return final_failed_urls;
}

fn get_url(base:&str,game_id: i32) -> String {
    return format!("{}{}", base, game_id);
}

fn latin1_to_string(s: &[u8]) -> String {
    s.iter().map(|&c| c as char).collect()
}

async fn decodeAndSave(response: Response,from_url: String) {
    // Log the response
    // println!("{:?}", response);
    // Get the body and write it to a file
    let body = response.text().await.unwrap();
    // println!("{:?}", body);
    // Base64 decode the body and catch various errors
    let decoded = base64::decode(&body).unwrap();
    // Read the first 32 bytes of the decoded body as uint32 and log it
    let jsonLengthSlice = &decoded[..4];
    // Make jsonLength a u32
    let jsonLength = u32::from_be_bytes([jsonLengthSlice[0], jsonLengthSlice[1], jsonLengthSlice[2], jsonLengthSlice[3]]);
    // Log the length of the JSON
    println!("{:?} characters ({})", jsonLength, from_url);
    // Read jsonLength characters from decoded after the first 4 bytes
    let jsonU8 = &decoded[4..(jsonLength+4) as usize];
    
    // Log the json bytes as string latin1
    let latinJson = latin1_to_string(jsonU8);
   // println!("{:?} - ({})", latinJson, from_url);
   
    // JSON decode the decoded body
    let jsonString: Value = serde_json::from_slice(latinJson.as_bytes()).unwrap();

    // Figure out if the json is coherent with what we expect
    // We want GS++ ranked ffa and teams games
    let gameMode = &jsonString["s"]["1"];
    let ranked = &jsonString["s"]["3"];
    let game_id = &jsonString["g"];

    if gameMode == "GS++" && ranked == true {
        saveReplayAndOriginal(game_id.to_string(), &jsonU8, &body)
    }
    println!("{} Not a GS++ ranked game", game_id);

}


fn saveReplayAndOriginal(game_id: String, jsonU8: &[u8], body: &String){
    let full_path_name: String = format!("./saves/{}.json",game_id);
    let full_path_original_base64: String = format!("./replays/{}",game_id);
    // Create /saves directory if it doesn't exist relative to the current directory
    // std::fs::create_dir_all("./saves").unwrap();
    let mut file = File::create(full_path_name).unwrap();
    // Write the jsonString to filename and create it if it doesn't exist
    file.write_all(jsonU8.to_vec().as_slice()).unwrap();
    // println!("Writing to file {}", game_id);

    let mut fileOriginal = File::create(full_path_original_base64).unwrap();
    // Write the original to /replays/filename.txt and create it if it doesn't exist
    fileOriginal.write_all(body.as_bytes()).unwrap();
}


    
