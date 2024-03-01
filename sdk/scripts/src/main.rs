use odra::types::casper_types::bytesrepr::FromBytes;
use odra::types::Address;
use odra::types::U128;
use odra::types::U256;
use odra::OdraType;
extern crate hex;

#[derive(OdraType, Eq, PartialEq, Debug)]
pub struct FeeTier {
    pub fee: Percentage,
    pub tick_spacing: u32,
}

#[derive(OdraType, Eq, PartialEq, Debug)]
pub struct Percentage {
    v: U128,
}

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
    // Fee tiers
    {
        let bytes = hex::decode("0800000046656554696572730100000007000000466565546965720a00000050657263656e7461676501640a000000").unwrap();

        // This is how you would normally deserialize bytes to InvariantConfig.
        // let (cfg, rem) = InvariantConfig::from_bytes(&bytes).unwrap();

        // This is how to do it manually.
        let (str, bytes) = String::from_bytes(&bytes).unwrap();
        println!("str: {:?}", str);
        println!("Bytes = {:?}", bytes);
        assert_eq!(str, "FeeTiers");
        let (count, mut bytes) = u32::from_bytes(bytes).unwrap();

        // let mut result = Vec::new();
        // for _ in 0..count {
        //     let (value, remainder) = FeeTier::from_bytes(bytes).unwrap();
        //     result.push(value);
        //     bytes = remainder;
        // }
        for _ in 0..count {
            let (value, remainder) = String::from_bytes(bytes).unwrap();
            println!("Value = {:?}", value);
            let (value, remainder) = String::from_bytes(remainder).unwrap();
            println!("Value = {:?}", value);
            println!("Remainder = {:?}", remainder);
            let (value, remainder) = U128::from_bytes(remainder).unwrap();
            println!("Value = {:?}", value);
            println!("Remainder = {:?}", remainder);
            let (value, remainder) = u32::from_bytes(remainder).unwrap();
            println!("Value = {:?}", value);
            println!("Remainder = {:?}", remainder);
        }
        // println!("result: {:?}", result);
        // assert!(bytes.is_empty());
    }
}
