#![allow(clippy::assign_op_pattern)]
#![allow(clippy::ptr_offset_with_cast)]
#![allow(clippy::manual_range_contains)]

use borsh::{BorshDeserialize, BorshSerialize};
use uint::construct_uint;

construct_uint! {
    #[derive(BorshSerialize, BorshDeserialize)]
    pub struct U192T(3);
}

construct_uint! {
    #[derive(BorshSerialize, BorshDeserialize)]
    pub struct U384T(6);
}

construct_uint! {
    #[derive(BorshSerialize, BorshDeserialize)]
    pub struct U448T(7);
}
