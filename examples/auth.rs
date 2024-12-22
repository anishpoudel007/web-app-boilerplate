use hmac::{Hmac, Mac};
use sha2::Sha256;

fn main() {
    let mut mac: Hmac<Sha256> =
        Hmac::new_from_slice(b"secret and secure key").expect("HMAC can take key of any size");

    mac.update(b"message for user");

    let result = mac.finalize();
    let code_bytes = result.into_bytes();

    let hex_code = hex::encode(code_bytes);

    // check validation
    let is_verified = verify(hex_code.as_ref(), "message for user");
    println!("Text is verified: {}", is_verified);
}

fn verify(hex_code: &str, text: &str) -> bool {
    let mut mac: Hmac<Sha256> =
        Hmac::new_from_slice(b"secret and secure key").expect("HMAC can take key of any size");

    mac.update(text.as_bytes());
    let result = mac.finalize();

    let code_byte = hex::decode(hex_code).unwrap();

    code_byte[..] == result.into_bytes()[..]
}
