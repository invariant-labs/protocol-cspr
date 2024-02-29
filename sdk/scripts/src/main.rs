use odra::types::casper_types::bytesrepr::FromBytes;
use odra::types::Address;
use odra::types::U256;
extern crate hex;

fn main() {
    let bytes = hex::decode("0f000000496e76617269616e74436f6e6669670012f4ba2d63b4610a48ed81620d62379f758ec5c33f862b53c9dbe9170023a5610a00000050657263656e7461676500").unwrap();

    // This is how you would normally deserialize bytes to InvariantConfig.
    // let (cfg, rem) = InvariantConfig::from_bytes(&bytes).unwrap();

    // This is how to do it manually.
    let (str, bytes) = String::from_bytes(&bytes).unwrap();
    assert_eq!(str, "InvariantConfig");
    let (admin, bytes) = Address::from_bytes(&bytes).unwrap();
    println!("admin: {:?}", admin);
    let (str, bytes) = String::from_bytes(&bytes).unwrap();
    assert_eq!(str, "Percentage");
    let (v, bytes) = U256::from_bytes(&bytes).unwrap();
    println!("v: {:?}", v);

    // Check if all bytes were used.
    assert!(bytes.is_empty());
}
