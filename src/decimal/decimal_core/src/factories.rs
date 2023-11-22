use alloc::string::ToString;
use quote::quote;

use crate::utils::string_to_ident;
use crate::DecimalCharacteristics;

pub fn generate_factories(characteristics: DecimalCharacteristics) -> proc_macro::TokenStream {
    let DecimalCharacteristics {
        struct_name,
        underlying_type,
        scale,
        big_type,
        ..
    } = characteristics;

    let name_str = &struct_name.to_string();
    let underlying_str = &underlying_type.to_string();

    let module_name = string_to_ident("tests_factories_", &name_str);

    proc_macro::TokenStream::from(quote!(

        impl Factories for #struct_name
        {
            type U = #underlying_type;

            fn from_integer(integer: Self::U) -> Self {
                Self::new(
                    integer.checked_mul(
                        Self::one().get()
                    ).unwrap_or_else(|| core::panic!("decimal: overflow while adjusting scale in method {}::from_integer()", #name_str))
                )
            }

            fn from_scale(integer: Self::U, scale: u8)-> Self {
                Self::new(
                    if #scale > scale {
                        let multiplier: #underlying_type = #underlying_type::from(10).checked_pow(#underlying_type::from((#scale - scale))).unwrap();
                        integer.checked_mul(multiplier).unwrap()
                    } else {
                        let denominator: #underlying_type = #underlying_type::from(10).checked_pow(#underlying_type::from((scale - #scale))).unwrap();
                        integer.checked_div(denominator).unwrap()
                    }
                )
            }

            fn checked_from_scale(integer: Self::U, scale: u8) -> core::result::Result<Self, alloc::string::String> {
                Ok(Self::new(
                    if #scale > scale {
                        let multiplier: #underlying_type = #underlying_type::from(10).checked_pow(#underlying_type::from((#scale - scale))).ok_or_else(|| "checked_from_scale: delta scale overflow")?;
                        integer.checked_mul(multiplier).ok_or_else(|| "checked_from_scale: (multiplier * base) overflow")?
                    } else {
                        let denominator: #underlying_type = #underlying_type::from(10).checked_pow(#underlying_type::from((scale - #scale))).ok_or_else(|| "checked_from_scale: delta scale overflow")?;
                        integer.checked_div(denominator).ok_or_else(|| "checked_from_scale: (base / denominator) overflow")?
                    }
                ))
            }

            fn from_scale_up(integer: Self::U, scale: u8) -> Self {
                Self::new(
                    if #scale > scale {
                        let multiplier: #underlying_type = #underlying_type::from(10).checked_pow(#underlying_type::from((#scale - scale))).unwrap();
                        integer.checked_mul(multiplier).unwrap()
                    } else {
                        let denominator: #underlying_type = #underlying_type::from(10).checked_pow(#underlying_type::from((scale - #scale))).unwrap();
                        integer.checked_add(denominator.checked_sub(#underlying_type::from(1u8)).unwrap()).unwrap()
                            .checked_div(denominator).unwrap()
                    }
                )
            }
        }

        // impl<T: Decimal> BetweenDecimals<T> for #struct_name
        // where
        //     Self: Factories<T::U>,
        // {
        //     fn from_decimal(other: T) -> Self {
        //         Self::from_scale(other.get(), T::scale())
        //     }

        //     fn checked_from_decimal(other: T) -> core::result::Result<Self, alloc::string::String> {
        //         Self::checked_from_scale(other.get(), T::scale())
        //     }

        //     fn from_decimal_up(other: T) -> Self {
        //         Self::from_scale_up(other.get(), T::scale())
        //     }
        // }

        impl<T> FactoriesToValue<T, #big_type> for #struct_name
        where
        T: Default
            + AsRef<[u64]>
            + From<u64>
            + core::ops::Shl<usize, Output = T>
            + core::ops::BitOrAssign,
        #big_type: Default
            + AsRef<[u64]>
            + From<u64>
            + core::ops::Shl<usize, Output = #big_type>
            + core::ops::BitOrAssign,
        {
            fn checked_from_scale_to_value(val: T, scale: u8) -> core::result::Result<#big_type, alloc::string::String> {
                let base: #big_type = #struct_name::from_value(val);

                Ok(if #scale > scale {
                    let multiplier: u128 = 10u128.checked_pow((#scale - scale) as u32).ok_or_else(|| "checked_from_scale_to_value: multiplier overflow")?;
                    base.checked_mul(multiplier.try_into().unwrap()).unwrap()
                } else {
                    let denominator: u128 = 10u128.checked_pow((scale - #scale) as u32).ok_or_else(|| "checked_from_scale_to_value: denominator overflow")?;
                    base.checked_div(denominator.try_into().unwrap()).unwrap()
                })
            }
        }

        impl<T: Decimal, #big_type> BetweenDecimalsToValue<T, #big_type> for #struct_name
        where
            Self: FactoriesToValue<T::U, #big_type>,
        {
            fn checked_from_decimal_to_value(other: T) -> core::result::Result<#big_type, alloc::string::String> {
                Self::checked_from_scale_to_value(other.get(), T::scale())
            }
        }

        #[cfg(test)]
        pub mod #module_name {
            use super::*;

            #[test]
            fn test_from_integer() {
                assert_eq!(
                    #struct_name::from_integer(#underlying_type::from(0)),
                    #struct_name::new(#underlying_type::from(0u8))
                );
            }

            #[test]
            fn test_from_scale() {
                assert_eq!(
                    #struct_name::from_scale(#underlying_type::from(0), 0),
                    #struct_name::new(#underlying_type::from(0u8))
                );
                assert_eq!(
                    #struct_name::from_scale_up(#underlying_type::from(0), 0),
                    #struct_name::new(#underlying_type::from(0u8))
                );

                assert_eq!(
                    #struct_name::from_scale(#underlying_type::from(0), 3),
                    #struct_name::new(#underlying_type::from(0u8))
                );
                assert_eq!(
                    #struct_name::from_scale_up(#underlying_type::from(0), 3),
                    #struct_name::new(#underlying_type::from(0u8))
                );

                assert_eq!(
                    #struct_name::from_scale(#underlying_type::from(42), #scale),
                    #struct_name::new(#underlying_type::from(42u8))
                );
                assert_eq!(
                    #struct_name::from_scale_up(#underlying_type::from(42), #scale),
                    #struct_name::new(#underlying_type::from(42u8))
                );

                assert_eq!(
                    #struct_name::from_scale(#underlying_type::from(42), #scale + 1),
                    #struct_name::new(#underlying_type::from(4u8))
                );
                assert_eq!(
                    #struct_name::from_scale_up(#underlying_type::from(42), #scale + 1),
                    #struct_name::new(#underlying_type::from(5u8))
                );
            }

            #[test]
            fn test_checked_from_scale() {
                assert_eq!(
                    #struct_name::checked_from_scale(#underlying_type::from(0), 0).unwrap(),
                    #struct_name::new(#underlying_type::from(0u8))
                );

                assert_eq!(
                    #struct_name::checked_from_scale(#underlying_type::from(0), 3).unwrap(),
                    #struct_name::new(#underlying_type::from(0u8))
                );

                assert_eq!(
                    #struct_name::checked_from_scale(#underlying_type::from(42), #scale).unwrap(),
                    #struct_name::new(#underlying_type::from(42u8))
                );

                assert_eq!(
                    #struct_name::checked_from_scale(#underlying_type::from(42), #scale + 1).unwrap(),
                    #struct_name::new(#underlying_type::from(4u8))
                );

                let max_u128: u128 = u128::MAX;
                assert_eq!(
                    #struct_name::checked_from_scale(#underlying_type::from(max_u128), 100_000).is_err(),
                    true
                );
            }

            #[test]
            fn test_checked_from_scale_to_value() {
                let big_zero = #big_type::from(0u8);
                let underlaying_zero = #underlying_type::from(0u8);

                let result = #struct_name::checked_from_scale_to_value(underlaying_zero, 0).unwrap();
                assert_eq!(result, big_zero);

                let result = #struct_name::checked_from_scale_to_value(underlaying_zero, 3).unwrap();
                assert_eq!(result, big_zero);

                let result = #struct_name::checked_from_scale_to_value(#underlying_type::from(42u8), #scale).unwrap();
                assert_eq!(result, #big_type::from(42u8));

                let result = #struct_name::checked_from_scale_to_value(#underlying_type::from(42u8), #scale + 1).unwrap();
                assert_eq!(result, #big_type::from(4u8));

                let max_val = #struct_name::max_value();
                assert_eq!(
                    #struct_name::checked_from_scale_to_value(max_val, 100_000).is_err(),
                    true
                );

                let result = #struct_name::checked_from_scale_to_value(#underlying_type::from(1u8), 38).unwrap();
                assert_eq!(result, big_zero);
            }

            #[test]
            fn test_checked_from_decimal_to_value() {
                let result = #struct_name::checked_from_decimal_to_value(#struct_name::new(#underlying_type::from(1u8))).unwrap();
                assert_eq!(result, #big_type::from(1u8));

                let result = #struct_name::checked_from_decimal_to_value(#struct_name::new(#underlying_type::from(42u8))).unwrap();
                assert_eq!(result, #big_type::from(42u8));
            }
        }
    ))
}
