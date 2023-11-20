use alloc::string::ToString;
use quote::quote;
use alloc::vec::Vec;

use crate::utils::string_to_ident;
use crate::DecimalCharacteristics;
pub fn generate_big_ops(characteristics: DecimalCharacteristics) -> proc_macro::TokenStream {
    let DecimalCharacteristics {
        struct_name,
        big_type,
        underlying_type,
        ..
    } = characteristics;

    let name_str = &struct_name.to_string();
    let underlying_str = &underlying_type.to_string();
    let big_str = &big_type.to_string();

    let module_name = string_to_ident("tests_big_ops_", &name_str);
    proc_macro::TokenStream::from(quote!(
        impl<T: Decimal> BigOps<T> for #struct_name
        where
        T: Decimal + alloc::fmt::Debug + Conversion,
        T::U: AsRef<[u64]>,
        {
            fn big_mul(self, rhs: T) -> Self {
                let self_len: usize = self.get().as_ref().len();

                let big_self: #big_type = self.cast::<#big_type>();
                let big_rhs: #big_type = rhs.cast::<#big_type>();
                let big_one: #big_type = T::one().cast::<#big_type>();

                let result = (big_self
                    .checked_mul(big_rhs)
                    .unwrap_or_else(|| core::panic!("decimal: lhs value can't fit into `{}` type in {}::big_mul()", #big_str, #name_str))
                    .checked_div(big_one)
                    .unwrap_or_else(|| core::panic!("decimal: overflow in method {}::big_mul()", #name_str))
                );

                let mut result_bytes: alloc::vec::Vec<u64> = result.as_ref().try_into().unwrap();
                let (self_result_bytes, remaining_bytes) = result_bytes.split_at_mut(self_len);

                if remaining_bytes.iter().any(|&x| x != 0) {
                    core::panic!("decimal: overflow casting result to `{}` type in method {}::big_mul()", #underlying_str, #name_str);
                }

                let mut result = #underlying_type::default();

                for (index, &value) in self_result_bytes.iter().enumerate() {
                    result |= #underlying_type::from(value) << (index * 64);
                }

                Self::new(result)
            }

            fn big_mul_up(self, rhs: T) -> Self {
                let mut self_bytes: alloc::vec::Vec<u64> = self.get().as_ref().try_into().unwrap();
                let mut rhs_bytes: alloc::vec::Vec<u64> = rhs.get().as_ref().try_into().unwrap();
                let mut rhs_one_bytes: alloc::vec::Vec<u64> = T::one().get().as_ref().try_into().unwrap();
                let mut rhs_almost_one_bytes: alloc::vec::Vec<u64> = T::almost_one().get().as_ref().try_into().unwrap();

                let self_len = self_bytes.len();
                let big_type_len: usize = #big_type::default().as_ref().len();

                self_bytes.resize(big_type_len,0);
                rhs_bytes.resize(big_type_len,0);
                rhs_one_bytes.resize(big_type_len,0);
                rhs_almost_one_bytes.resize(big_type_len,0);

                let big_self: #big_type = #big_type(self_bytes.try_into().unwrap());
                let big_rhs: #big_type = #big_type(rhs_bytes.try_into().unwrap());
                let big_one: #big_type = #big_type(rhs_one_bytes.try_into().unwrap());
                let big_almost_one: #big_type = #big_type(rhs_almost_one_bytes.try_into().unwrap());

                let result = big_self
                    .checked_mul(big_rhs)
                    .unwrap_or_else(|| core::panic!("decimal: overflow in method {}::big_mul_up()", #name_str))
                    .checked_add(big_almost_one)
                    .unwrap_or_else(|| core::panic!("decimal: overflow in method {}::big_mul_up()", #name_str))
                    .checked_div(big_one)
                    .unwrap_or_else(|| core::panic!("decimal: overflow in method {}::big_mul_up()", #name_str));

                let mut result_bytes: alloc::vec::Vec<u64> = result.as_ref().try_into().unwrap();
                let (self_result_bytes, remaining_bytes) = result_bytes.split_at_mut(self_len);

                if remaining_bytes.iter().any(|&x| x != 0) {
                    core::panic!("decimal: overflow casting result to `{}` type in method {}::big_mul()", #underlying_str, #name_str);
                }

                let mut result = #underlying_type::default();

                for (index, &value) in self_result_bytes.iter().enumerate() {
                    result |= #underlying_type::from(value) << (index * 64);
                }

                Self::new(result)
            }

            fn big_div(self, rhs: T) -> Self {
                let mut self_bytes: alloc::vec::Vec<u64> = self.get().as_ref().try_into().unwrap();
                let mut rhs_bytes: alloc::vec::Vec<u64> = rhs.get().as_ref().try_into().unwrap();
                let mut rhs_one_bytes: alloc::vec::Vec<u64> = T::one().get().as_ref().try_into().unwrap();

                let self_len = self_bytes.len();
                let big_type_len: usize = #big_type::default().as_ref().len();

                self_bytes.resize(big_type_len,0);
                rhs_bytes.resize(big_type_len,0);
                rhs_one_bytes.resize(big_type_len,0);

                let big_self: #big_type = #big_type(self_bytes.try_into().unwrap());
                let big_rhs: #big_type = #big_type(rhs_bytes.try_into().unwrap());
                let big_one: #big_type = #big_type(rhs_one_bytes.try_into().unwrap());

                let result = (big_self
                    .checked_mul(big_one)
                    .unwrap_or_else(|| core::panic!("decimal: lhs value can't fit into `{}` type in {}::big_div()", #big_str, #name_str))
                    .checked_div(big_rhs)
                    .unwrap_or_else(|| core::panic!("decimal: overflow in method {}::big_div()", #name_str))
                );

                let mut result_bytes: alloc::vec::Vec<u64> = result.as_ref().try_into().unwrap();
                let (self_result_bytes, remaining_bytes) = result_bytes.split_at_mut(self_len);

                if remaining_bytes.iter().any(|&x| x != 0) {
                    core::panic!("decimal: overflow casting result to `{}` type in method {}::big_div()", #underlying_str, #name_str);
                }

                let mut result = #underlying_type::default();

                for (index, &value) in self_result_bytes.iter().enumerate() {
                    result |= #underlying_type::from(value) << (index * 64);
                }

                Self::new(result)

            }
            fn checked_big_div(self, rhs: T) -> core::result::Result<Self, alloc::string::String> {


                let mut self_bytes: alloc::vec::Vec<u64> = self.get().as_ref().try_into()
                    .map_err(|_| alloc::format!("decimal: lhs value can't fit into `{}` type in {}::checked_big_div()", #big_str, #name_str))?;
                let mut rhs_bytes: alloc::vec::Vec<u64> = rhs.get().as_ref().try_into()
                    .map_err(|_| alloc::format!("decimal: rhs value can't fit into `{}` type in {}::checked_big_div()", #big_str, #name_str))?;
                let mut rhs_one_bytes: alloc::vec::Vec<u64> = T::one().get().as_ref().try_into()
                    .map_err(|_| alloc::format!("decimal: rhs::one value can't fit into `{}` type in {}::checked_big_div()", #big_str, #name_str))?;

                let self_len = self_bytes.len();
                let big_type_len: usize = #big_type::default().as_ref().len();

                self_bytes.resize(big_type_len,0);
                rhs_bytes.resize(big_type_len,0);
                rhs_one_bytes.resize(big_type_len,0);

                let big_self: #big_type = #big_type(self_bytes.try_into()
                    .map_err(|_| alloc::format!("decimal: lhs value can't fit into `{}` type in {}::checked_big_div()", #big_str, #name_str))?);
                let big_rhs: #big_type = #big_type(rhs_bytes.try_into()
                    .map_err(|_| alloc::format!("decimal: rhs value can't fit into `{}` type in {}::checked_big_div()", #big_str, #name_str))?);
                let big_one: #big_type = #big_type(rhs_one_bytes.try_into()
                    .map_err(|_| alloc::format!("decimal: rhs::one value can't fit into `{}` type in {}::checked_big_div()", #big_str, #name_str))?);

                let result = (big_self
                    .checked_mul(big_one)
                    .ok_or_else(|| alloc::format!("decimal: overflow in method {}::checked_big_div()", #name_str))?
                    .checked_div(big_rhs)
                    .ok_or_else(|| alloc::format!("decimal: overflow in method {}::checked_big_div()", #name_str))?
                );

                let mut result_bytes: alloc::vec::Vec<u64> = result.as_ref().try_into().unwrap();
                let (self_result_bytes, remaining_bytes) = result_bytes.split_at_mut(self_len);

                if remaining_bytes.iter().any(|&x| x != 0) {
                    return Err(alloc::format!("decimal: overflow casting result to `{}` type in method {}::checked_big_div()", #underlying_str, #name_str));
                }

                let mut result = #underlying_type::default();

                for (index, &value) in self_result_bytes.iter().enumerate() {
                    result |= #underlying_type::from(value) << (index * 64);
                }

                Ok(Self::new(result))
            }

            fn big_div_up(self, rhs: T) -> Self {
                let mut self_bytes: alloc::vec::Vec<u64> = self.get().as_ref().try_into().unwrap();
                let mut rhs_bytes: alloc::vec::Vec<u64> = rhs.get().as_ref().try_into().unwrap();
                let mut rhs_one_bytes: alloc::vec::Vec<u64> = T::one().get().as_ref().try_into().unwrap();

                let self_len = self_bytes.len();
                let big_type_len: usize = #big_type::default().as_ref().len();

                self_bytes.resize(big_type_len,0);
                rhs_bytes.resize(big_type_len,0);
                rhs_one_bytes.resize(big_type_len,0);

                let big_self: #big_type = #big_type(self_bytes.try_into().unwrap());
                let big_rhs: #big_type = #big_type(rhs_bytes.try_into().unwrap());
                let big_one: #big_type = #big_type(rhs_one_bytes.try_into().unwrap());

                let result = (big_self
                    .checked_mul(big_one)
                    .unwrap_or_else(|| core::panic!("decimal: lhs value can't fit into `{}` type in {}::big_div_up()", #big_str, #name_str))
                    .checked_add(big_rhs)
                    .unwrap_or_else(|| core::panic!("decimal: overflow in method {}::big_div_up()", #name_str))
                    .checked_sub(#big_type::from(1))
                    .unwrap_or_else(|| core::panic!("decimal: underflow in method {}::big_div_up()", #name_str))
                    .checked_div(big_rhs)
                    .unwrap_or_else(|| core::panic!("decimal: overflow in method {}::big_div_up()", #name_str))
                );

                let mut result_bytes: alloc::vec::Vec<u64> = result.as_ref().try_into().unwrap();
                let (self_result_bytes, remaining_bytes) = result_bytes.split_at_mut(self_len);

                if remaining_bytes.iter().any(|&x| x != 0) {
                    core::panic!("decimal: overflow casting result to `{}` type in method {}::big_div_up()", #underlying_str, #name_str);
                }

                let mut result = #underlying_type::default();

                for (index, &value) in self_result_bytes.iter().enumerate() {
                    result |= #underlying_type::from(value) << (index * 64);
                }

                Self::new(result)
            }
        }

        #[cfg(test)]
        pub mod #module_name {
            use super::*;

            #[test]
            fn test_big_mul () {
                let a = #struct_name::new(#underlying_type::from(2));
                let b = #struct_name::one();
                assert_eq!(a.big_mul(b), #struct_name::new(#underlying_type::from(2)));
            }

            #[test]
            fn test_big_mul_up () {
                let a = #struct_name::new(#underlying_type::from(2));
                let b = #struct_name::one();
                assert_eq!(a.big_mul_up(b), a);
            }

            #[test]
            fn test_big_div () {
                let a = #struct_name::new(#underlying_type::from(2));
                let b = #struct_name::one();
                assert_eq!(a.big_div(b), #struct_name::new(#underlying_type::from(2)));
            }

            #[test]
            fn test_checked_big_div () {
                let a = #struct_name::new(#underlying_type::from(29));
                let b = #struct_name::one();
                assert_eq!(a.big_div(b), a);
            }

            #[test]
            fn test_big_div_up () {
                let a = #struct_name::new(#underlying_type::from(2));
                let b = #struct_name::one();
                assert_eq!(a.big_div_up(b), #struct_name::new(#underlying_type::from(2)));
            }
        }
    ))
}
