use alloc::string::ToString;
use quote::quote;

use crate::utils::string_to_ident;
use crate::DecimalCharacteristics;

pub fn generate_by_number(characteristics: DecimalCharacteristics) -> proc_macro::TokenStream {
    let DecimalCharacteristics {
        struct_name,
        big_type,
        underlying_type,
        ..
    } = characteristics;

    let name_str = &struct_name.to_string();
    let underlying_str = &underlying_type.to_string();

    let module_name = string_to_ident("tests_by_number_", &name_str);

    proc_macro::TokenStream::from(quote!(
            impl ByNumber<#big_type> for #struct_name {
                fn big_div_by_number(self, rhs: #big_type) -> Self {
                    let mut self_bytes: alloc::vec::Vec<u64> = self.get().as_ref().try_into().unwrap();
                    let mut self_one_bytes: alloc::vec::Vec<u64> = Self::one().get().as_ref().try_into().unwrap();

                    let self_len = self_bytes.len();
                    let big_type_len: usize = #big_type::default().as_ref().len();

                    self_bytes.resize(big_type_len, 0);
                    self_one_bytes.resize(big_type_len, 0);

                    let big_self: #big_type = #big_type(self_bytes.try_into().unwrap());
                    let big_self_one: #big_type = #big_type(self_one_bytes.try_into().unwrap());

                    let result = (big_self
                        .checked_mul(big_self_one)
                        .unwrap()
                        .checked_div(rhs)
                        .unwrap()
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

                fn checked_big_div_by_number(self, rhs: #big_type) -> core::result::Result<Self, alloc::string::String> {
                    let mut self_bytes: alloc::vec::Vec<u64> = self.get().as_ref().try_into().unwrap();
                    let mut self_one_bytes: alloc::vec::Vec<u64> = Self::checked_one().unwrap().get().as_ref().try_into().unwrap();

                    let self_len = self_bytes.len();
                    let big_type_len: usize = #big_type::default().as_ref().len();

                    self_bytes.resize(big_type_len, 0);
                    self_one_bytes.resize(big_type_len, 0);

                    let big_self: #big_type = #big_type(self_bytes.try_into().unwrap());
                    let big_self_one: #big_type = #big_type(self_one_bytes.try_into().unwrap());

                    let result = (big_self
                        .checked_mul(big_self_one)
                        .unwrap()
                        .checked_div(rhs)
                        .unwrap()
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

                    Ok(Self::new(result))

                }

                fn big_div_by_number_up(self, rhs: #big_type) -> Self {
                    let mut self_bytes: alloc::vec::Vec<u64> = self.get().as_ref().try_into().unwrap();
                    let mut self_one_bytes: alloc::vec::Vec<u64> = Self::one().get().as_ref().try_into().unwrap();

                    let self_len = self_bytes.len();
                    let big_type_len: usize = #big_type::default().as_ref().len();

                    self_bytes.resize(big_type_len, 0);
                    self_one_bytes.resize(big_type_len, 0);

                    let big_self: #big_type = #big_type(self_bytes.try_into().unwrap());
                    let big_self_one: #big_type = #big_type(self_one_bytes.try_into().unwrap());

                    let result = (big_self
                        .checked_mul(big_self_one)
                        .unwrap()
                        .checked_add(rhs.checked_sub(#big_type::from(1u8)).unwrap())
                        .unwrap()
                        .checked_div(rhs)
                        .unwrap()
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

                fn checked_big_div_by_number_up(self, rhs: #big_type) -> core::result::Result<Self, alloc::string::String> {
                    let mut self_bytes: alloc::vec::Vec<u64> = self.get().as_ref().try_into().unwrap();
                    let mut self_one_bytes: alloc::vec::Vec<u64> = Self::checked_one().unwrap().get().as_ref().try_into().unwrap();

                    let self_len = self_bytes.len();
                    let big_type_len: usize = #big_type::default().as_ref().len();

                    self_bytes.resize(big_type_len, 0);
                    self_one_bytes.resize(big_type_len, 0);

                    let big_self: #big_type = #big_type(self_bytes.try_into().unwrap());
                    let big_self_one: #big_type = #big_type(self_one_bytes.try_into().unwrap());

                    let result = (big_self
                        .checked_mul(big_self_one)
                        .unwrap()
                        .checked_add(rhs.checked_sub(#big_type::from(1u8)).unwrap())
                        .unwrap()
                        .checked_div(rhs)
                        .unwrap()
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

                    Ok(Self::new(result))
    }
            }

            impl<T: Decimal> ToValue<T, #big_type> for #struct_name
            where
                T::U: AsRef<[u64]>,
            {
                fn big_mul_to_value(self, rhs: T) -> #big_type {
                    let mut self_bytes: alloc::vec::Vec<u64> = self.get().as_ref().try_into().unwrap();
                    let mut rhs_bytes: alloc::vec::Vec<u64> = rhs.get().as_ref().try_into().unwrap();
                    let mut rhs_one_bytes: alloc::vec::Vec<u64> = T::one().get().as_ref().try_into().unwrap();

                    let self_len = self_bytes.len();
                    let big_type_len: usize = #big_type::default().as_ref().len();

                    self_bytes.resize(big_type_len, 0);
                    rhs_bytes.resize(big_type_len, 0);
                    rhs_one_bytes.resize(big_type_len, 0);

                    let big_self: #big_type = #big_type(self_bytes.try_into().unwrap());
                    let big_rhs: #big_type = #big_type(rhs_bytes.try_into().unwrap());
                    let big_rhs_one: #big_type = #big_type(rhs_one_bytes.try_into().unwrap());

                    let result = (big_self
                        .checked_mul(big_rhs)
                        .unwrap()
                        .checked_div(big_rhs_one)
                        .unwrap()
                    );

                    let mut result_bytes: alloc::vec::Vec<u64> = result.as_ref().try_into().unwrap();
                    let big_result: #big_type = #big_type(result_bytes.try_into().unwrap());

                    big_result

                    }

                fn big_mul_to_value_up(self, rhs: T) -> #big_type {
                    let mut self_bytes: alloc::vec::Vec<u64> = self.get().as_ref().try_into().unwrap();
                    let mut rhs_bytes: alloc::vec::Vec<u64> = rhs.get().as_ref().try_into().unwrap();
                    let mut rhs_almost_one_bytes: alloc::vec::Vec<u64> = T::almost_one().get().as_ref().try_into().unwrap();
                    let mut rhs_one_bytes: alloc::vec::Vec<u64> = T::one().get().as_ref().try_into().unwrap();

                    let self_len = self_bytes.len();
                    let big_type_len: usize = #big_type::default().as_ref().len();

                    self_bytes.resize(big_type_len, 0);
                    rhs_bytes.resize(big_type_len, 0);
                    rhs_almost_one_bytes.resize(big_type_len, 0);
                    rhs_one_bytes.resize(big_type_len, 0);

                    let big_self: #big_type = #big_type(self_bytes.try_into().unwrap());
                    let big_rhs: #big_type = #big_type(rhs_bytes.try_into().unwrap());
                    let big_rhs_almost_one: #big_type = #big_type(rhs_almost_one_bytes.try_into().unwrap());
                    let big_rhs_one: #big_type = #big_type(rhs_one_bytes.try_into().unwrap());

                    let result = (big_self
                        .checked_mul(big_rhs)
                        .unwrap()
                        .checked_add(big_rhs_almost_one)
                        .unwrap()
                        .checked_div(big_rhs_one)
                        .unwrap()
                    );

                    let mut result_bytes: alloc::vec::Vec<u64> = result.as_ref().try_into().unwrap();
                    let big_result: #big_type = #big_type(result_bytes.try_into().unwrap());

                    big_result

                    }
            }

            #[cfg(test)]
            pub mod #module_name {
                use super::*;

                #[test]
                fn test_big_div_up_by_number () {
                    let a = #struct_name::new(#underlying_type::from(2u8));
                    let mut struct_one_bytes: alloc::vec::Vec<u64> = #struct_name::one().get().as_ref().try_into().unwrap();
                    struct_one_bytes.resize(#big_type::default().as_ref().len(), 0);
                    let b: #big_type = #big_type(struct_one_bytes.try_into().unwrap());
                    assert_eq!(a.big_div_by_number(b), #struct_name::new(#underlying_type::from(2u8)));
                    assert_eq!(a.big_div_by_number_up(b), #struct_name::new(#underlying_type::from(2u8)));
                }

                #[test]
                fn test_checked_big_div_by_number() {
                    let a = #struct_name::new(#underlying_type::from(2u8));
                    let mut struct_one_bytes: alloc::vec::Vec<u64> = #struct_name::one().get().as_ref().try_into().unwrap();
                    struct_one_bytes.resize(#big_type::default().as_ref().len(), 0);
                    let b: #big_type = #big_type(struct_one_bytes.try_into().unwrap());
                    assert_eq!(a.checked_big_div_by_number(b), Ok(#struct_name::new(#underlying_type::from(2u8))));
                }

                #[test]
                fn test_checked_big_div_by_number_up() {
                    let a = #struct_name::new(#underlying_type::from(2u8));
                    let mut struct_one_bytes: alloc::vec::Vec<u64> = #struct_name::one().get().as_ref().try_into().unwrap();
                    struct_one_bytes.resize(#big_type::default().as_ref().len(), 0);
                    let b: #big_type = #big_type(struct_one_bytes.try_into().unwrap());
                    assert_eq!(a.checked_big_div_by_number_up(b), Ok(#struct_name::new(#underlying_type::from(2u8))));
                }

                #[test]
                fn test_big_mul_to_value () {
                    let a = #struct_name::new(#underlying_type::from(2u8));
                    let b = #struct_name::one();
                    let mut a_bytes: alloc::vec::Vec<u64> = a.get().as_ref().try_into().unwrap();
                    a_bytes.resize(#big_type::default().as_ref().len(), 0);
                    let c: #big_type = #big_type(a_bytes.try_into().unwrap());
                    let mut b_bytes: alloc::vec::Vec<u64> = b.get().as_ref().try_into().unwrap();
                    b_bytes.resize(#big_type::default().as_ref().len(), 0);
                    let d: #big_type = #big_type(b_bytes.try_into().unwrap());
                    assert_eq!(a.big_mul_to_value(b), c);
                    assert_eq!(a.big_mul_to_value_up(b), c);
                }
            }
        ))
}
