use std::collections::HashMap;

use std::fs::read;
use std::path::Path;

use serde_json::Value;

fn read_file_as_bytes<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<u8>> {
    read(path)
}

fn latin1_to_utf8(bytes: &[u8]) -> String {
    bytes.iter().map(|&c| c as char).collect()
}
fn parse_json_from_str(json_str: &str) -> serde_json::Result<Value> {
    serde_json::from_str(json_str)
}

fn atob_unicode(encoded: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Decode the base64 string into bytes
    let bytes = base64::decode(encoded)?;

    // Interpret bytes as UTF-16 and collect into a String
    let utf16_string = if bytes.len() % 2 == 0 {
        // Safe to convert bytes to u16s because we checked that the length is even
        let u16_slice = unsafe {
            let ptr = bytes.as_ptr() as *const u16;
            std::slice::from_raw_parts(ptr, bytes.len() / 2)
        };
        String::from_utf16(u16_slice)?
    } else {
        return Err("Byte stream is of odd length and cannot be converted to UTF-16.".into());
    };

    Ok(utf16_string)
}

fn main() -> std::io::Result<()> {
    // Load filelist from /saves
    let filelist = std::fs::read_dir("./saves").unwrap();

    // Loop through each file in the filelist

    // Prepare an key value pair to store the player id and the players names
    let mut players: HashMap<String, Vec<String>> = HashMap::new();



Ok(for file in filelist {
    // Get the filename
    let filename = file.unwrap().file_name().into_string().unwrap();
    // Split the filename by '.' and get the first element
    let game_id = filename.split(".").collect::<Vec<&str>>()[0].parse::<i32>().unwrap();
    // Print the game_id
    //println!("Game ID: {}", game_id);
    // Load the file contents
    // let file_contents_read = std::fs::read_to_string();
    let path = format!("./saves/{}", filename);
        // Step 1: Read the file as bytes
        let bytes = read_file_as_bytes(path)?;

        // Step 2: Convert Latin1 bytes to a UTF-8 String
        let utf8_string = latin1_to_utf8(&bytes);
    
        // Step 3: Parse the string as JSON
        match parse_json_from_str(&utf8_string) {
            Ok(json) => {},
            Err(e) => eprintln!("Failed to parse JSON: {:?}", e),
        }


    // Unwrap the file contents or get the error message
    // let file_contents = match file_contents_read {
    //     Ok(contents) => contents,
    //     Err(e) => {
    //         println!("Error reading file: {:?}, [ Game: {} ]", e, game_id);
    //         continue;
    //     }
    // };

    // Parse as json
    let json: serde_json::Value = serde_json::from_str(&utf8_string).unwrap();
    // Loop on json["t"] to list all the players
    for player in json["t"].as_array().unwrap() {
        let playerObj = &player["p"][0]["d"];
        let id = &playerObj["0"];
        //println!("ID: {:?}", id);

        // If the player is not in the hashmap, add it
        // 310540 < has u1
        let mut playerName = &playerObj["1"];
        // Check playerName exists and is not Null

        let playerNameStr = match playerName.as_str() {
            Some(name) => name.to_string(),
            None => {
                let u1 = &player["p"][0]["u1"];
                    // It's a base64, decode it
                    let decoded = base64::decode(u1.as_str().unwrap()).unwrap();
                    // Convert to unicode16
                    //let playerNameStrU16 = decoded.iter().map(|&c| c as char).collect::<String>();

                    let playerNameStrU16 = atob_unicode(u1.as_str().unwrap()).unwrap();

                    // Print the unicode16
                    //println!("PlayerUnicoded: {:?}", playerNameStrU16);
                    playerNameStrU16
            }
        };

        let mut current = players.get_mut(&id.to_string());

        if let Some(current_vec) = current {
            // current_vec est maintenant une référence mutable au vecteur si la clé existe
            if current_vec.contains(&playerNameStr) {
                // Log duplicate
                //println!("Duplicate: {:?}", playerNameStr);
            } else {
                // Log inserted
               
                current_vec.push(playerNameStr);
                println!("Found rename: {:?}", current_vec);
            }
        } else {
            // Log inserted
            // println!("Inserted: {:?}", playerNameStr);
            players.insert(id.to_string(), vec![playerNameStr]);
        }


    }
})

}