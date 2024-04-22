
use base64::decode;


fn main() {

    fn latin1_to_string(s: &[u8]) -> String {
        s.iter().map(|&c| c as char).collect()
    }

    let badUtf = std::fs::read_to_string("src/fixtures/badUtf.txt").unwrap();
    // Print
    println!("{:?}", badUtf);
    // Read file ./fixtures/badB64.txt relative to project root
    let b64Text = std::fs::read_to_string("src/fixtures/badUtf.txt").unwrap();
    // Base 64 decode b64Text

    let base64Decoded = base64::decode(b64Text).unwrap();
    println!("{:?}", latin1_to_string(&base64Decoded));


    

}