use borsh::{BorshDeserialize, BorshSerialize};

use uint::construct_uint;
construct_uint! {
    #[derive(BorshSerialize, BorshDeserialize)]
    pub struct U64T(1);
}

construct_uint! {
    #[derive(BorshSerialize, BorshDeserialize)]
    pub struct U128T(2);
}

construct_uint! {
    #[derive(BorshSerialize, BorshDeserialize)]
    pub struct U192T(3);
}

construct_uint! {
    #[derive(BorshSerialize, BorshDeserialize)]
    pub struct U256T(4);
}

construct_uint! {
    #[derive(BorshSerialize, BorshDeserialize)]
    pub struct U320T(5);
}

construct_uint! {
    #[derive(BorshSerialize, BorshDeserialize)]
    pub struct U384T(6);
}

construct_uint! {
    #[derive(BorshSerialize, BorshDeserialize)]
    pub struct U448T(7);
}

construct_uint! {
    #[derive(BorshSerialize, BorshDeserialize)]
    pub struct U512T(8);
}
