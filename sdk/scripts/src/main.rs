use blake2b_simd::{Hash, Params};
use odra::types::casper_types::bytesrepr::FromBytes;
use odra::types::casper_types::bytesrepr::ToBytes;
use odra::types::casper_types::ContractPackageHash;
use odra::types::Address;
use odra::types::U128;
use odra::types::U256;
use odra::OdraType;

extern crate hex;

#[derive(OdraType, Debug, PartialEq)]
pub enum InvariantError {
    InvalidTickSpacing,
    TokensAreSame,
    InvalidFee,
}

#[derive(OdraType, Eq, PartialEq, Debug)]
pub struct Percentage {
    v: U128,
}

#[derive(OdraType, Eq, PartialEq, Debug)]
pub struct PoolKey {
    pub token_x: Address,
    pub token_y: Address,
    pub fee_tier: FeeTier,
}

impl Default for PoolKey {
    fn default() -> Self {
        Self {
            token_x: Address::Contract(ContractPackageHash::from([0x0; 32])),
            token_y: Address::Contract(ContractPackageHash::from([0x0; 32])),
            fee_tier: FeeTier::default(),
        }
    }
}

impl PoolKey {
    pub fn new(
        token_0: Address,
        token_1: Address,
        fee_tier: FeeTier,
    ) -> Result<Self, InvariantError> {
        if token_0 == token_1 {
            return Err(InvariantError::TokensAreSame);
        }

        if token_0 < token_1 {
            Ok(PoolKey {
                token_x: token_0,
                token_y: token_1,
                fee_tier,
            })
        } else {
            Ok(PoolKey {
                token_x: token_1,
                token_y: token_0,
                fee_tier,
            })
        }
    }
}

#[derive(OdraType, Eq, PartialEq, Debug)]
pub struct FeeTier {
    pub fee: Percentage,
    pub tick_spacing: u32,
}

impl Default for FeeTier {
    fn default() -> Self {
        Self {
            fee: Percentage { v: U128::from(0) },
            tick_spacing: 0,
        }
    }
}

fn main() {
    // {
    //     println!("Decoding config");
    //     let bytes = hex::decode("0f000000496e76617269616e74436f6e6669670012f4ba2d63b4610a48ed81620d62379f758ec5c33f862b53c9dbe9170023a5610a00000050657263656e7461676500").unwrap();
    //     println!("Bytes = {:?}", bytes);
    //     // This is how you would normally deserialize bytes to InvariantConfig.
    //     // let (cfg, rem) = InvariantConfig::from_bytes(&bytes).unwrap();

    //     // This is how to do it manually.
    //     let (str, bytes) = String::from_bytes(&bytes).unwrap();
    //     assert_eq!(str, "InvariantConfig");
    //     println!("str = {:?}", str);
    //     println!("Rest of the bytes = {:?}", bytes);
    //     println!("-------------------");
    //     let (admin, bytes) = Address::from_bytes(&bytes).unwrap();
    //     println!("admin: {:?}", admin);
    //     println!("Rest of the bytes = {:?}", bytes);
    //     println!("-------------------");
    //     let (str, bytes) = String::from_bytes(&bytes).unwrap();
    //     assert_eq!(str, "Percentage");
    //     println!("Rest of the bytes = {:?}", bytes);
    //     println!("-------------------");
    //     let (v, bytes) = U256::from_bytes(&bytes).unwrap();
    //     println!("v: {:?}", v);

    //     // Check if all bytes were used.
    //     assert!(bytes.is_empty());
    // }
    // {
    //     println!("Decoding config");
    //     let bytes = hex::decode("09000000546d705374727563740163").unwrap();

    //     // This is how you would normally deserialize bytes to InvariantConfig.
    //     // let (cfg, rem) = InvariantConfig::from_bytes(&bytes).unwrap();

    //     // This is how to do it manually.
    //     let (str, bytes) = String::from_bytes(&bytes).unwrap();
    //     println!("str: {:?}", str);
    //     assert_eq!(str, "TmpStruct");
    //     println!("Bytes = {:?}", bytes);
    //     // let (admin, bytes) = Address::from_bytes(&bytes).unwrap();
    //     // println!("admin: {:?}", admin);
    //     // println!("Bytes = {:?}", bytes);
    //     // let (str, bytes) = String::from_bytes(&bytes).unwrap();
    //     // assert_eq!(str, "Percentage");
    //     // println!("Bytes = {:?}", bytes);
    //     let (v, bytes) = U256::from_bytes(&bytes).unwrap();
    //     println!("v: {:?}", v);

    //     // Check if all bytes were used.
    //     // assert!(bytes.is_empty());
    // }
    // // Fee tiers
    // {
    //     println!("Fee Tiers");
    //     let bytes = hex::decode("0800000046656554696572730100000007000000466565546965720a00000050657263656e7461676501640a000000").unwrap();

    //     // This is how you would normally deserialize bytes to InvariantConfig.
    //     // let (cfg, rem) = InvariantConfig::from_bytes(&bytes).unwrap();

    //     // This is how to do it manually.
    //     let (str, bytes) = String::from_bytes(&bytes).unwrap();
    //     println!("str: {:?}", str);
    //     println!("Bytes = {:?}", bytes);
    //     assert_eq!(str, "FeeTiers");
    //     let (count, mut bytes) = u32::from_bytes(bytes).unwrap();

    //     // let mut result = Vec::new();
    //     // for _ in 0..count {
    //     //     let (value, remainder) = FeeTier::from_bytes(bytes).unwrap();
    //     //     result.push(value);
    //     //     bytes = remainder;
    //     // }
    //     for _ in 0..count {
    //         let (value, remainder) = String::from_bytes(bytes).unwrap();
    //         println!("Value = {:?}", value);
    //         let (value, remainder) = String::from_bytes(remainder).unwrap();
    //         println!("Value = {:?}", value);
    //         println!("Remainder = {:?}", remainder);
    //         let (value, remainder) = U128::from_bytes(remainder).unwrap();
    //         println!("Value = {:?}", value);
    //         println!("Remainder = {:?}", remainder);
    //         let (value, remainder) = u32::from_bytes(remainder).unwrap();
    //         println!("Value = {:?}", value);
    //         println!("Remainder = {:?}", remainder);
    //     }
    //     // println!("result: {:?}", result);
    //     // assert!(bytes.is_empty());
    // }
    // // Mapping
    // {
    //     println!("Mapping in parent contract");
    //     // Create empty buffer.
    //     let mut buffor: Vec<u8> = Vec::new();

    //     let token_0 = Address::Contract(ContractPackageHash::from([0x01; 32]));
    //     let token_1 = Address::Contract(ContractPackageHash::from([0x02; 32]));

    //     println!("Token 0 = {:?}", token_0);
    //     println!("Token 1 = {:?}", token_1);

    //     let fee_tier = FeeTier {
    //         fee: Percentage { v: U128::from(100) },
    //         tick_spacing: 10,
    //     };

    //     println!("Fee Tier = {:?}", fee_tier);

    //     let pool_key = PoolKey::new(token_0, token_1, fee_tier).unwrap();

    //     buffor.extend_from_slice(b"pools");
    //     buffor.extend_from_slice(&pool_key.to_bytes().unwrap());

    //     println!("Buffor: {:?}", buffor);

    //     // Hash the buffer using Blake2b.
    //     let result = Params::new()
    //         .hash_length(32) // Output hash length in bytes
    //         .to_state()
    //         .update(&buffor)
    //         .finalize();

    //     // Convert the hash to hex.
    //     println!("Blake2b: {:?}", result);
    //     let encoded = hex::encode(result.as_bytes());
    //     // 8993177b688dbcd454730d11f28d54508151536789928beb4deff08cc5a3e786
    //     println!("Invariant Pool Hex encoded: {:?}", encoded);
    // }
    // Decode mapping result
    {
        println!("Decoding mapping Pool result");
        let bytes = hex::decode("0104000000506f6f6c090000004c697175696469747900090000005371727450726963650a000000a1edccce1bc2d3000000000900000046656547726f777468000900000046656547726f777468000b000000546f6b656e416d6f756e74000b000000546f6b656e416d6f756e740000c033098e01000000c033098e010000006796ab4158be14efcb3db532e3311123925a2a24f2add0d93eda0f396e4aee5f00").unwrap();
        // Some("Pool")
        let (value, bytes): (Option<String>, &[u8]) = Option::from_bytes(&bytes).unwrap();
        println!("Value = {:?}", value);

        // Liquidity
        let (value, bytes) = String::from_bytes(&bytes).unwrap();
        println!("Struct Name = {:?}", value);

        // Liquidity Value
        let (value, bytes) = U256::from_bytes(&bytes).unwrap();
        println!("Value = {:?}", value);

        // SqrtPrice
        let (value, bytes) = String::from_bytes(&bytes).unwrap();
        println!("Struct Name = {:?}", value);

        // SqrtPrice Value
        let (value, bytes) = U256::from_bytes(&bytes).unwrap();
        println!("Value = {:?}", value);

        // Current tick index
        let (value, bytes) = i32::from_bytes(&bytes).unwrap();
        println!("Current tick index = {:?}", value);

        // Fee growth Global X struct
        let (value, bytes) = String::from_bytes(&bytes).unwrap();
        println!("Struct Name = {:?}", value);

        // Fee Growth Global X value
        let (value, bytes) = U256::from_bytes(&bytes).unwrap();
        println!("Value = {:?}", value);

        // Fee growth Global Y Struct
        let (value, bytes) = String::from_bytes(&bytes).unwrap();
        println!("Struct Name = {:?}", value);

        // Fee Growth Global Y Value
        let (value, bytes) = U256::from_bytes(&bytes).unwrap();
        println!("Value = {:?}", value);

        // Fee Protocol Token Y
        let (value, bytes) = String::from_bytes(&bytes).unwrap();
        println!("Struct Name = {:?}", value);

        // Fee Protocol Token Y Value
        let (value, bytes) = U256::from_bytes(&bytes).unwrap();
        println!("Value = {:?}", value);

        // Fee Protocol Token Y
        let (value, bytes) = String::from_bytes(&bytes).unwrap();
        println!("Struct Name = {:?}", value);

        // Fee Protocol Token Y Value
        let (value, bytes) = U256::from_bytes(&bytes).unwrap();
        println!("Value = {:?}", value);

        // Start timestamp
        let (value, bytes) = u64::from_bytes(&bytes).unwrap();
        println!("Starting timestamp = {:?}", value);

        // Last timestamp
        let (value, bytes) = u64::from_bytes(&bytes).unwrap();
        println!("Last timestamp = {:?}", value);

        // Fee Receiver
        let (value, bytes) = Address::from_bytes(&bytes).unwrap();
        println!("Value = {:?}", value);

        println!("Bytes = {:?}", bytes);
        // Oracle initliazed
        let (value, bytes) = bool::from_bytes(&bytes).unwrap();
        println!("Oracle initialized = {:?}", value);

        // Check if all bytes were used.
        assert!(bytes.is_empty());
    }
    // // Mapping
    // {
    //     println!("Mapping in nested contract");
    //     // Create empty buffer.
    //     let mut buffor: Vec<u8> = Vec::new();

    //     let token_0 = Address::Contract(ContractPackageHash::from([0x01; 32]));
    //     let token_1 = Address::Contract(ContractPackageHash::from([0x02; 32]));

    //     let fee_tier = FeeTier {
    //         fee: Percentage { v: U128::from(100) },
    //         tick_spacing: 10,
    //     };

    //     let pool_key = PoolKey::new(token_0, token_1, fee_tier).unwrap();

    //     buffor.extend_from_slice(b"nested_pools");
    //     buffor.extend_from_slice(b"pools");
    //     buffor.extend_from_slice(&pool_key.to_bytes().unwrap());

    //     // Hash the buffer using Blake2b.
    //     let result = Params::new()
    //         .hash_length(32) // Output hash length in bytes
    //         .to_state()
    //         .update(&buffor)
    //         .finalize();

    //     // Convert the hash to hex.
    //     println!("Blake2b: {:?}", result);
    //     let encoded = hex::encode(result.as_bytes());
    //     // 8993177b688dbcd454730d11f28d54508151536789928beb4deff08cc5a3e786
    //     println!("Nested Pool Hex encoded: {:?}", encoded);
    // }
}
