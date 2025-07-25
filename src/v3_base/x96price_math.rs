use alloy::primitives::{U256, U512};

pub fn compute_amount_possible(
    from0: bool,
        available_liquidity: &U256,
    current_sqrt_price: &U256,
    next_sqrt_price: &U256,
) -> Option<U256> {
    println!("computing amount possible");
    // Q96 = 2^96
    let q96: U512 = U512::ONE << 96;

    //promote everything to U512
    let liq: U512 = U512::from(*available_liquidity);
    let cur: U512 = U512::from(*current_sqrt_price);
    let nxt: U512 = U512::from(*next_sqrt_price);

    if from0 {
        // Δx = L·(√P_next − √P_curr)·Q96 ÷ (√P_curr·√P_next)
        println!("from 0");
        println!(
            "next price {}",
            next_sqrt_price
        );
        println!(
            "curr price {}",
            current_sqrt_price
        );

        let diff = nxt.checked_sub(cur)?;
        println!("passed");

        if diff.is_zero() {
            println!("diff is zezo");
            return None;
        }

        println!(
            "q 96 {}",
            q96
        );
        println!(
            "available liquidity {}",
            available_liquidity
        );
        println!(
            "diff {}",
            diff
        );
        // numerator = L * diff * Q96
        let impact = liq.checked_mul(diff)?;
        let numerator: U512 =
            U512::from(impact).checked_mul(q96)?;

        // denominator = cur * nxt
        let denominator = cur.checked_mul(nxt)?;

        let res =
            U256::from(numerator.checked_div(denominator)?);

        Some(res)
    } else {
        // Δy = L·(√P_curr − √P_next) ÷ Q96
        let diff = cur.checked_sub(nxt)?;
        println!(
            "diff {}",
            diff
        );
        if diff.is_zero() {
            println!("diff is zero");
            return None;
        }

        let numerator = liq.checked_mul(diff)?;
        println!(
            "numerator {}",
            numerator
        );
        Some(U256::from(numerator.checked_div(q96)?))
    }
}
/// Given Δy (token1 amount) and liquidity L, compute the next √P
pub fn compute_price_from0(
    amount: &U256,
    available_liquidity: &U256,
    current_sqrt_price: &U256,
    add: bool,
) -> Option<U256> {
    // Debug prints (optional)
    // println!("Inputs:");
    // println!("  Δx (amount): {}", amount);
    // println!("  L (liquidity): {}", available_liquidity);
    // println!("  √P (current_sqrt_price): {}", current_sqrt_price);
    // Step 1: Compute L << 96 (Q96L)
    let q96_l =
        *available_liquidity << (U256::from(96_u32));
    // println!("Q96L (L << 96): {}", Q96L);

    // Step 2: Compute (L << 96) / √P (scaled_liquidity)
    let scaled_liquidity = q96_l
        .checked_div(U256::from(*current_sqrt_price))?;
    // println!("scaled_liquidity (Q96L / √P): {}", scaled_liquidity);

    // Step 3: Compute denominator = scaled_liquidity ± Δx
    let denominator = if add {
        scaled_liquidity.checked_add(*amount)?
    } else {
        scaled_liquidity.checked_sub(*amount)?
    };
    // println!("denominator (scaled_liquidity ± Δx): {}", denominator);

    // Step 4: Compute new_sqrt_price = Q96L / denominator
    let new_sqrt_price = q96_l.checked_div(denominator)?;
    // println!("new_sqrt_price (Q96L / denominator): {}", new_sqrt_price);

    Some(new_sqrt_price)
} // Given Δy (token1 amount) and liquidity L, compute the next √P
/// note: everything in Q96 fixed‐point (i.e. <<96) internally
pub fn compute_price_from1(
    amount: &U256,
    available_liquidity: &U256,
    current_sqrt_price: &U256,
    add: bool,
) -> Option<U256> {
    // Q96 = 2^96
    let q96 = U256::ONE << 96;
    // 1) Scale amount into Q96:   Δy * Q96
    let dy_q96 = amount.checked_mul(q96)?;
    // 2) Divide by liquidity:    Δ√P = (Δy·Q96) / L
    let liquidity_u256 = U256::from(*available_liquidity);
    let delta_sqrt = dy_q96.checked_div(liquidity_u256)?;
    // 3) Apply to current √P
    let cur: U256 = U256::from(*current_sqrt_price);
    let next = if add {
        cur.checked_add(delta_sqrt)?
    } else {
        cur.checked_sub(delta_sqrt)?
    };
    Some(next)
}
pub fn update_liquidity(
    current_liquidity: U256,
    liquidity_net: i128,
) -> Option<U256> {
    if liquidity_net < 0 {
        // If liquidity_net is negative, it means liquidity is removed.
        // We need to subtract the absolute value of liquidity_net.
        let abs_net =
            U256::from(liquidity_net.abs() as u128); // Convert abs(i128) to u128 then U256
        current_liquidity.checked_sub(abs_net)
    } else {
        // If liquidity_net is positive or zero, it means liquidity is added.
        let pos_net = U256::from(liquidity_net as u128); // Convert positive i128 to u128 then U256
        current_liquidity.checked_add(pos_net)
    }
}
