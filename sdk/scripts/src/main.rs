use blake2b_simd::Params;
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
    {
        println!("Decoding config");
        let bytes = hex::decode("0f000000496e76617269616e74436f6e6669670012f4ba2d63b4610a48ed81620d62379f758ec5c33f862b53c9dbe9170023a5610a00000050657263656e7461676500").unwrap();
        println!("Bytes = {:?}", bytes);
        // This is how you would normally deserialize bytes to InvariantConfig.
        // let (cfg, rem) = InvariantConfig::from_bytes(&bytes).unwrap();

        // This is how to do it manually.
        let (str, bytes) = String::from_bytes(&bytes).unwrap();
        assert_eq!(str, "InvariantConfig");
        println!("str = {:?}", str);
        let (admin, bytes) = Address::from_bytes(&bytes).unwrap();
        println!("admin: {:?}", admin);
        let (str, bytes) = String::from_bytes(&bytes).unwrap();
        assert_eq!(str, "Percentage");
        let (v, bytes) = U256::from_bytes(&bytes).unwrap();
        println!("v: {:?}", v);

        // Check if all bytes were used.
        assert!(bytes.is_empty());
    }
    // Fee tiers
    {
        println!("Fee Tiers");
        let bytes = hex::decode("0800000046656554696572730100000007000000466565546965720a00000050657263656e7461676501640a000000").unwrap();

        let (str, bytes) = String::from_bytes(&bytes).unwrap();
        println!("str: {:?}", str);
        println!("Bytes = {:?}", bytes);
        assert_eq!(str, "FeeTiers");
        let (count, bytes) = u32::from_bytes(bytes).unwrap();

        // Alternative way
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
    }

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

        // Oracle initliazed
        let (value, bytes) = bool::from_bytes(&bytes).unwrap();
        println!("Value = {:?}", value);

        assert!(bytes.is_empty());
    }
    // Mapping
    {
        println!("Mapping in nested contract");
        let mut buffor: Vec<u8> = Vec::new();

        let token_0 = Address::Contract(ContractPackageHash::from([0x01; 32]));
        let token_1 = Address::Contract(ContractPackageHash::from([0x02; 32]));

        let fee_tier = FeeTier {
            fee: Percentage { v: U128::from(100) },
            tick_spacing: 10,
        };

        let pool_key = PoolKey::new(token_0, token_1, fee_tier.clone()).unwrap();

        buffor.extend_from_slice(b"pools");
        buffor.extend_from_slice(b"#");
        buffor.extend_from_slice(b"pools");
        buffor.extend_from_slice(&pool_key.to_bytes().unwrap());

        // Hash the buffer using Blake2b.
        let result = Params::new()
            .hash_length(32) // Output hash length in bytes
            .to_state()
            .update(&buffor)
            .finalize();

        // Convert the hash to hex.
        let encoded = hex::encode(result.as_bytes());
        println!("Hash = {}", encoded);
        // 8993177b688dbcd454730d11f28d54508151536789928beb4deff08cc5a3e786
    }
    // Decode pool Keys
    {
        println!("Decoding pool keys");
        let bytes = hex::decode("08000000506f6f6c4b6579730100000007000000506f6f6c4b657901c34b7847a3fe4d5d12e4975b4eddfed10d25f0cb165d740a4a74606172d7c47201da1b9f07767375414fc7649ac8719be5d7104f49bc8c030bd51c45b0dbb2290807000000466565546965720a00000050657263656e7461676501370a000000").unwrap();

        let (str, bytes) = String::from_bytes(&bytes).unwrap();
        println!("str: {:?}", str);
        let (count, bytes) = u32::from_bytes(bytes).unwrap();
        println!("count: {:?}", count);
        let (str, bytes) = String::from_bytes(&bytes).unwrap();
        println!("str: {:?}", str);
        let (token_0, bytes) = Address::from_bytes(&bytes).unwrap();
        println!("token_0: {:?}", token_0);
        let (token_1, bytes) = Address::from_bytes(&bytes).unwrap();
        println!("token_1: {:?}", token_1);
        let (str, bytes) = String::from_bytes(&bytes).unwrap();
        println!("str: {:?}", str);
        let (str, bytes) = String::from_bytes(&bytes).unwrap();
        println!("str: {:?}", str);
        let (fee, bytes) = U128::from_bytes(&bytes).unwrap();
        println!("fee: {:?}", fee);
        let (tick_spacing, bytes) = u32::from_bytes(&bytes).unwrap();
        println!("tick_spacing: {:?}", tick_spacing);
        println!("Reamingin bytes length = {:?}", bytes.len())
    }
    // Serialize Pool key
    {
        println!("Serializing pool key");
        let token_0 = Address::Contract(ContractPackageHash::from([0x01; 32]));
        let token_1 = Address::Contract(ContractPackageHash::from([0x02; 32]));

        let fee_tier = FeeTier {
            fee: Percentage { v: U128::from(100) },
            tick_spacing: 10,
        };

        let pool_key = PoolKey::new(token_0, token_1, fee_tier.clone()).unwrap();
        let pool_key_bytes = pool_key.to_bytes().unwrap();

        let mut buffor: Vec<u8> = vec![];

        let pool_key_struct_bytes = "PoolKey".to_bytes().unwrap();
        let token_0_bytes = token_0.to_bytes().unwrap();
        let token_1_bytes = token_1.to_bytes().unwrap();
        let fee_tier_struct_bytes = "FeeTier".to_bytes().unwrap();
        let tick_spacing_bytes = fee_tier.tick_spacing.to_bytes().unwrap();
        let fee_bytes = fee_tier.fee.to_bytes().unwrap();

        buffor.extend_from_slice(&pool_key_struct_bytes);
        buffor.extend_from_slice(&token_0_bytes);
        buffor.extend_from_slice(&token_1_bytes);
        buffor.extend_from_slice(&fee_tier_struct_bytes);
        buffor.extend_from_slice(&fee_bytes);
        buffor.extend_from_slice(&tick_spacing_bytes);

        assert_eq!(buffor, pool_key_bytes);
    }
}
