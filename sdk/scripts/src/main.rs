use odra::types::casper_types::bytesrepr::FromBytes;
use odra::types::Address;
use odra::types::U256;
extern crate hex;

fn main() {
    {
        let bytes = hex::decode("0f000000496e76617269616e74436f6e6669670012f4ba2d63b4610a48ed81620d62379f758ec5c33f862b53c9dbe9170023a5610a00000050657263656e7461676500").unwrap();
        println!("Bytes = {:?}", bytes);
        // This is how you would normally deserialize bytes to InvariantConfig.
        // let (cfg, rem) = InvariantConfig::from_bytes(&bytes).unwrap();

        // This is how to do it manually.
        let (str, bytes) = String::from_bytes(&bytes).unwrap();
        assert_eq!(str, "InvariantConfig");
        println!("str = {:?}", str);
        println!("Rest of the bytes = {:?}", bytes);
        println!("-------------------");
        let (admin, bytes) = Address::from_bytes(&bytes).unwrap();
        println!("admin: {:?}", admin);
        println!("Rest of the bytes = {:?}", bytes);
        println!("-------------------");
        let (str, bytes) = String::from_bytes(&bytes).unwrap();
        assert_eq!(str, "Percentage");
        println!("Rest of the bytes = {:?}", bytes);
        println!("-------------------");
        let (v, bytes) = U256::from_bytes(&bytes).unwrap();
        println!("v: {:?}", v);

        // Check if all bytes were used.
        assert!(bytes.is_empty());
    }
    {
        let bytes = hex::decode("09000000546d705374727563740163").unwrap();

        // This is how you would normally deserialize bytes to InvariantConfig.
        // let (cfg, rem) = InvariantConfig::from_bytes(&bytes).unwrap();

        // This is how to do it manually.
        let (str, bytes) = String::from_bytes(&bytes).unwrap();
        println!("str: {:?}", str);
        assert_eq!(str, "TmpStruct");
        println!("Bytes = {:?}", bytes);
        // let (admin, bytes) = Address::from_bytes(&bytes).unwrap();
        // println!("admin: {:?}", admin);
        // println!("Bytes = {:?}", bytes);
        // let (str, bytes) = String::from_bytes(&bytes).unwrap();
        // assert_eq!(str, "Percentage");
        // println!("Bytes = {:?}", bytes);
        let (v, bytes) = U256::from_bytes(&bytes).unwrap();
        println!("v: {:?}", v);

        // Check if all bytes were used.
        // assert!(bytes.is_empty());
    }
}
