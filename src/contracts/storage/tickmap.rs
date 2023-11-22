use super::PoolKey;
use crate::math::sqrt_price::calculate_sqrt_price;
use crate::math::sqrt_price::SqrtPrice;
use crate::math::MAX_TICK;
use odra::Mapping;

pub const TICK_SEARCH_RANGE: i32 = 256;
pub const CHUNK_SIZE: i32 = 64;

#[odra::module]
pub struct Tickmap {
    pub bitmap: Mapping<(u16, PoolKey), u64>,
}

pub fn tick_to_position(tick: i32, tick_spacing: u32) -> (u16, u8) {
    assert!(
        tick >= -MAX_TICK && tick <= MAX_TICK,
        "tick not in range of <{}, {}>",
        -MAX_TICK,
        MAX_TICK
    );

    assert_eq!(
        (tick % tick_spacing as i32),
        0,
        "tick not divisible by tick spacing"
    );

    let bitmap_index = (tick + MAX_TICK) / tick_spacing as i32;

    let chunk: u16 = (bitmap_index / CHUNK_SIZE) as u16;
    let bit: u8 = (bitmap_index % CHUNK_SIZE) as u8;

    (chunk, bit)
}

fn get_bit_at_position(value: u64, position: u8) -> u64 {
    (value >> position) & 1
}

fn flip_bit_at_position(value: u64, position: u8) -> u64 {
    value ^ (1 << position)
}

pub fn get_search_limit(tick: i32, tick_spacing: u32, up: bool) -> i32 {
    let index = tick / tick_spacing as i32;

    // limit unscaled
    let limit = if up {
        // search range is limited to 256 at the time ...
        let range_limit = index + TICK_SEARCH_RANGE;
        // ...also ticks for sqrt_prices over 2^64 aren't needed
        let sqrt_price_limit = MAX_TICK / tick_spacing as i32;

        range_limit.min(sqrt_price_limit)
    } else {
        let range_limit = index - TICK_SEARCH_RANGE;
        let sqrt_price_limit = -MAX_TICK / tick_spacing as i32;

        range_limit.max(sqrt_price_limit)
    };

    // scaled by tick_spacing
    limit * tick_spacing as i32
}

#[odra::module]
impl Tickmap {
    pub fn next_initialized(&self, tick: i32, tick_spacing: u32, pool_key: PoolKey) -> Option<i32> {
        let limit = get_search_limit(tick, tick_spacing, true);

        if tick + tick_spacing as i32 > MAX_TICK {
            return None;
        }

        // add 1 to not check current tick
        let (mut chunk, mut bit) =
            tick_to_position(tick.checked_add(tick_spacing as i32)?, tick_spacing);
        let (limiting_chunk, limiting_bit) = tick_to_position(limit, tick_spacing);

        while chunk < limiting_chunk || (chunk == limiting_chunk && bit <= limiting_bit) {
            let mut shifted = self.bitmap.get(&(chunk, pool_key.clone())).unwrap_or(0) >> bit;

            if shifted != 0 {
                while shifted.checked_rem(2)? == 0 {
                    shifted >>= 1;
                    bit = bit.checked_add(1)?;
                }

                return if chunk < limiting_chunk || (chunk == limiting_chunk && bit <= limiting_bit)
                {
                    // no possibility of overflow
                    let index: i32 = (chunk as i32 * CHUNK_SIZE) + bit as i32;

                    Some(
                        index
                            .checked_sub(MAX_TICK / tick_spacing as i32)?
                            .checked_mul(i32::try_from(tick_spacing).ok()?)?,
                    )
                } else {
                    None
                };
            }

            // go to the text chunk
            // if let value = chunk.checked_add(1)? {
            if let Some(value) = chunk.checked_add(1) {
                chunk = value;
            } else {
                return None;
            }
            bit = 0;
        }

        None
    }

    // tick_spacing - spacing already scaled by tick_spacing
    pub fn prev_initialized(&self, tick: i32, tick_spacing: u32, pool_key: PoolKey) -> Option<i32> {
        // don't subtract 1 to check the current tick
        let limit = get_search_limit(tick, tick_spacing, false); // limit scaled by tick_spacing
        let (mut chunk, mut bit) = tick_to_position(tick as i32, tick_spacing);
        let (limiting_chunk, limiting_bit) = tick_to_position(limit, tick_spacing);

        while chunk > limiting_chunk || (chunk == limiting_chunk && bit >= limiting_bit) {
            // always safe due to limitated domain of bit variable
            let mut mask = 1u128 << bit; // left = MSB direction (increase value)
            let value = self.bitmap.get(&(chunk, pool_key.clone())).unwrap_or(0) as u128;

            // enter if some of previous bits are initialized in current chunk
            if value.checked_rem(mask.checked_shl(1)?)? > 0 {
                // skip uninitalized ticks
                while value & mask == 0 {
                    mask >>= 1;
                    bit = bit.checked_sub(1)?;
                }

                // return first initalized tick if limiit is not exceeded, otherswise return None
                return if chunk > limiting_chunk || (chunk == limiting_chunk && bit >= limiting_bit)
                {
                    // no possibility to overflow
                    let index: i32 = (chunk as i32 * CHUNK_SIZE) + bit as i32;

                    Some(
                        index
                            .checked_sub(MAX_TICK / tick_spacing as i32)?
                            .checked_mul(i32::try_from(tick_spacing).ok()?)?,
                    )
                } else {
                    None
                };
            }

            // go to the next chunk
            // if let value = chunk.checked_sub(1)? {
            if let Some(value) = chunk.checked_sub(1) {
                chunk = value;
            } else {
                return None;
            }
            bit = CHUNK_SIZE as u8 - 1;
        }

        None
    }

    // Finds closes initialized tick in direction of trade
    // and compares its sqrt_price to the sqrt_price limit of the trade
    pub fn get_closer_limit(
        &self,
        sqrt_price_limit: SqrtPrice,
        x_to_y: bool,
        current_tick: i32,
        tick_spacing: u32,
        pool_key: PoolKey,
    ) -> (SqrtPrice, Option<(i32, bool)>) {
        let closes_tick_index = if x_to_y {
            self.prev_initialized(current_tick, tick_spacing, pool_key)
        } else {
            self.next_initialized(current_tick, tick_spacing, pool_key)
        };

        match closes_tick_index {
            Some(index) => {
                let sqrt_price = calculate_sqrt_price(index).unwrap();
                if x_to_y && sqrt_price > sqrt_price_limit {
                    (sqrt_price, Some((index, true)))
                } else if !x_to_y && sqrt_price < sqrt_price_limit {
                    (sqrt_price, Some((index, true)))
                } else {
                    (sqrt_price_limit, None)
                }
            }
            None => {
                let index = get_search_limit(current_tick, tick_spacing, !x_to_y);
                let sqrt_price = calculate_sqrt_price(index).unwrap();

                assert!(current_tick != index, "LimitReached");

                if x_to_y && sqrt_price > sqrt_price_limit {
                    (sqrt_price, Some((index, false)))
                } else if !x_to_y && sqrt_price < sqrt_price_limit {
                    (sqrt_price, Some((index, false)))
                } else {
                    (sqrt_price_limit, None)
                }
            }
        }
    }

    pub fn get(&self, tick: i32, tick_spacing: u32, pool_key: PoolKey) -> bool {
        let (chunk, bit) = tick_to_position(tick, tick_spacing);
        let returned_chunk = self.bitmap.get(&(chunk, pool_key)).unwrap_or(0);
        get_bit_at_position(returned_chunk, bit) == 1
    }

    pub fn flip(&mut self, value: bool, tick: i32, tick_spacing: u32, pool_key: PoolKey) {
        let (chunk, bit) = tick_to_position(tick, tick_spacing);
        let returned_chunk = self.bitmap.get(&(chunk, pool_key.clone())).unwrap_or(0);

        assert_eq!(
            get_bit_at_position(returned_chunk, bit) == 0,
            value,
            "tick initialize tick again"
        );

        self.bitmap.add(
            &(chunk, pool_key),
            flip_bit_at_position(returned_chunk, bit),
        );
    }
}
