# Fix for Validium Gateway Settlement Issue

## Problem Summary
When a gateway is running in **Validium mode**, chains trying to settle on it fail with the error: **"gas per pub data limit is zero"**

This affects both:
- Rollup ZK chains settling on a Validium Gateway ❌
- Validium ZK chains settling on a Validium Gateway ❌

## Root Cause Analysis

The issue occurs because:

1. **Validium gateways set `fair_pubdata_price` to 0** since they don't have real L1 pubdata costs
2. When chains query the gateway's fee history, they receive `l2_pubdata_price = 0`
3. The `gas_per_pubdata()` calculation returns 0 (l2_pubdata_price / base_fee = 0)
4. Transaction validation fails with "gas per pub data limit is zero" error

### Code Path:
1. `estimate_effective_pubdata_price()` returns 0 for `PubdataSendingMode::Custom` (Validium)
2. `cap_pubdata_fee()` also returns 0 for `L1BatchCommitmentMode::Validium`
3. This causes the gateway's blocks to have `fair_pubdata_price = 0`
4. Settling chains fail validation when `eip712_meta.gas_per_pubdata = 0`

## Solution

Modified two functions in `/core/node/fee_model/src/l1_gas_price/gas_adjuster/mod.rs`:

### 1. Fixed `estimate_effective_pubdata_price()`:
```rust
PubdataSendingMode::Custom => {
    // For Validium/Custom DA, we need to return a minimum gas per pubdata price
    // to ensure that transactions settling on this gateway can calculate a valid
    // gas_per_pubdata value. Without this, chains trying to settle on a Validium
    // gateway will fail with "gas per pub data limit is zero" error.
    // We use REQUIRED_L2_GAS_PRICE_PER_PUBDATA (800) as the minimum.
    const MIN_PUBDATA_PRICE_FOR_VALIDIUM: u64 = 800;
    MIN_PUBDATA_PRICE_FOR_VALIDIUM
}
```

### 2. Fixed `cap_pubdata_fee()`:
```rust
L1BatchCommitmentMode::Validium => {
    // For Validium mode, we still need to return the pubdata fee
    // to ensure chains settling on this gateway can function properly.
    // We don't cap it to 0 as that would break settlement transactions.
    if pubdata_fee > max_blob_base_fee as f64 {
        tracing::warn!("Pubdata fee for Validium is high: {pubdata_fee}, using max allowed: {max_blob_base_fee}");
        return max_blob_base_fee;
    }
    pubdata_fee as u64
}
```

## Impact

After this fix:
- Validium gateways will report a minimum `pubdata_price` of 800 (REQUIRED_L2_GAS_PRICE_PER_PUBDATA)
- Chains settling on Validium gateways will calculate valid `gas_per_pubdata` values
- Both Rollup and Validium chains can successfully settle on Validium gateways

## Additional Considerations

While this fix addresses the immediate issue, there may be other edge cases when using Validium gateways:

1. **Configuration**: Validium gateways should properly configure:
   - `pubdata_overhead_part = 0.0` (no pubdata overhead)
   - `compute_overhead_part = 1.0` (all overhead from compute)
   - `sender_pubdata_sending_mode = "Custom"`

2. **Future Improvements**: 
   - Consider making the minimum pubdata price configurable
   - Add better support for custom DA pricing models
   - Improve documentation for Validium gateway setup

## Files Modified

- `/core/node/fee_model/src/l1_gas_price/gas_adjuster/mod.rs` - Fixed pubdata pricing logic

This fix ensures that Validium gateways can be used as settlement layers without breaking the gas calculation logic for settling chains.
