use alloy::primitives::{I256, U256, U512, aliases::I24};

pub fn price_from_tick(target_tick: I24) -> Option<U256> {
    println!(
        "calculating price for tick: {}",
        target_tick
    );
    let max_tick: I24 = I24::try_from(887272).unwrap();
    let abs_tick = target_tick.abs();

    if abs_tick > max_tick {
        eprintln!(
            "[0] Tick {} exceeds maximum allowed (±{})",
            target_tick, max_tick
        );
        return None;
    }

    let mut sqrt_price_x128 =
        if (abs_tick & I24::ONE) != I24::ZERO {
            U512::from_str_radix(
                "fffcb933bd6fad37aa2d162d1a594001",
                16,
            )
            .unwrap()
        } else {
            U512::from(1) << 128
        };

    let magic_numbers = [
        // mask 0x1  (handled in your `sqrt_price_x128 = …` init)
        (
            0x2,
            U512::from_str_radix(
                "fff97272373d413259a46990580e213a",
                16,
            )
            .unwrap(),
        ),
        (
            0x4,
            U512::from_str_radix(
                "fff2e50f5f656932ef12357cf3c7fdcc",
                16,
            )
            .unwrap(),
        ),
        (
            0x8,
            U512::from_str_radix(
                "ffe5caca7e10e4e61c3624eaa0941cd0",
                16,
            )
            .unwrap(),
        ),
        (
            0x10,
            U512::from_str_radix(
                "ffcb9843d60f6159c9db58835c926644",
                16,
            )
            .unwrap(),
        ),
        (
            0x20,
            U512::from_str_radix(
                "ff973b41fa98c081472e6896dfb254c0",
                16,
            )
            .unwrap(),
        ),
        (
            0x40,
            U512::from_str_radix(
                "ff2ea16466c96a3843ec78b326b52861",
                16,
            )
            .unwrap(),
        ),
        (
            0x80,
            U512::from_str_radix(
                "fe5dee046a99a2a811c461f1969c3053",
                16,
            )
            .unwrap(),
        ),
        (
            0x100,
            U512::from_str_radix(
                "fcbe86c7900a88aedcffc83b479aa3a4",
                16,
            )
            .unwrap(),
        ),
        (
            0x200,
            U512::from_str_radix(
                "f987a7253ac413176f2b074cf7815e54",
                16,
            )
            .unwrap(),
        ),
        (
            0x400,
            U512::from_str_radix(
                "f3392b0822b70005940c7a398e4b70f3",
                16,
            )
            .unwrap(),
        ),
        (
            0x800,
            U512::from_str_radix(
                "e7159475a2c29b7443b29c7fa6e889d9",
                16,
            )
            .unwrap(),
        ),
        (
            0x1000,
            U512::from_str_radix(
                "d097f3bdfd2022b8845ad8f792aa5825",
                16,
            )
            .unwrap(),
        ),
        (
            0x2000,
            U512::from_str_radix(
                "a9f746462d870fdf8a65dc1f90e061e5",
                16,
            )
            .unwrap(),
        ),
        (
            0x4000,
            U512::from_str_radix(
                "70d869a156d2a1b890bb3df62baf32f7",
                16,
            )
            .unwrap(),
        ),
        (
            0x8000,
            U512::from_str_radix(
                "31be135f97d08fd981231505542fcfa6",
                16,
            )
            .unwrap(),
        ),
        (
            0x10000,
            U512::from_str_radix(
                "9aa508b5b7a84e1c677de54f3e99bc9",
                16,
            )
            .unwrap(),
        ),
        (
            0x20000,
            U512::from_str_radix(
                "5d6af8dedb81196699c329225ee604",
                16,
            )
            .unwrap(),
        ),
        (
            0x40000,
            U512::from_str_radix(
                "2216e584f5fa1ea926041bedfe98",
                16,
            )
            .unwrap(),
        ),
        (
            0x80000,
            U512::from_str_radix(
                "48a170391f7dc42444e8fa2",
                16,
            )
            .unwrap(),
        ),
    ];

    // Iterate from highest mask to lowest
    for (mask, magic) in magic_numbers.iter() {
        if abs_tick & I24::try_from(*mask).unwrap()
            != I24::ZERO
        {
            // wrap on overflow, then shift down
            let (wrapped, _) =
                sqrt_price_x128.overflowing_mul(*magic);
            sqrt_price_x128 = wrapped >> 128;
        }
    }
    let mut p256 = U256::from(sqrt_price_x128);

    if target_tick > I24::ZERO {
        if sqrt_price_x128.is_zero() {
            return None; // Should ideally not happen if initial sqrt_price_x128 is non-zero
        }
        p256 = U256::MAX
            .checked_div(p256)
            .unwrap();
    }

    // 4) shift down to Q128.96 and round up if any low bits remain

    let shifted = p256 >> 32;
    let sqrt_price_x96_u256: U256 = if p256
        & ((U256::ONE << 32) - U256::ONE)
        != U256::ZERO
    {
        shifted + U256::ONE
    } else {
        shifted
    };

    // 5) cast to U160
    // let sqrt_price_x96 = U160::from(sqrt_price_x96_u256);

    println!(
        "value: {}",
        sqrt_price_x96_u256.clone()
    );
    Some(sqrt_price_x96_u256)
}
// Convert a sqrt price Q128.96 to the nearest tick index (I24)
/// Port of Uniswap V3's TickMath.getTickAtSqrtRatio
pub fn tick_from_price(
    sqrt_price_x96: U256,
) -> Option<I24> {
    // Define bounds as U256 to avoid u128 overflow
    let min_sqrt = U256::from(4295128739u64);
    let max_sqrt = U256::from_str_radix(
        "1461446703485210103287273052203988822378723970342",
        10,
    )
    .unwrap();

    if sqrt_price_x96 < min_sqrt
        || sqrt_price_x96 >= max_sqrt
    {
        eprintln!(
            "Sqrt price {} out of bounds",
            sqrt_price_x96
        );
        return None;
    }
    /*
        println!(
            "calculating tick for price: {}",
            sqrt_price_x96
        );
    */

    // Convert to Q128.128 for log calculation
    let sqrroot_price_x128: U256 = sqrt_price_x96 << 32;

    // Compute log2(sqrroot_price_x128)
    let msb = 255 - sqrroot_price_x128.leading_zeros();
    println!(
        "most significant bit {}",
        msb
    );
    let mut log2: I256 = (I256::try_from(msb).unwrap()
        - I256::try_from(128u8).unwrap())
        << 64;

    let mut r = if msb >= 128 {
        sqrroot_price_x128 >> (msb - 127)
    } else {
        sqrroot_price_x128 << (127 - msb)
    };
    for i in 0..14 {
        r = (r * r) >> 127;
        let f: U256 = r >> 128;
        let shift = 63 - i;
        let a: U256 = f << shift;
        log2 |= I256::from(a);
        r >>= f;
    }

    let log_sqrt10001 = log2
        * I256::try_from("255738958999603826347141")
            .unwrap();
    let denom = I256::ONE << 128;
    let low = (log_sqrt10001
        - I256::try_from(
            "3402992956809132418596140100660247210",
        )
        .unwrap())
    .div_euclid(denom);
    let high = (log_sqrt10001
        + I256::try_from(
            "291339464771989622907027621153398088495",
        )
        .unwrap())
    .div_euclid(denom);
    println!(
        "high {}",
        high
    );
    println!(
        "low {}",
        low
    );

    // Calculate candidate ticks
    let tick_low: I24 = I24::from(low);
    let tick_high: I24 = I24::from(high);

    println!(
        "low: {} | high: {}",
        tick_low, tick_high
    );

    let result = if tick_high == tick_low {
        tick_high
    } else {
        if price_from_tick(tick_high)? >= sqrt_price_x96 {
            tick_high
        } else {
            tick_low
        }
    };
    Some(result)
}
