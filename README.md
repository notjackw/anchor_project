# Wasabi Technical 2 Interview

## Summary of changes (since Friday)
- Refactor: moved instruction handlers into separate modules, since `lib.rs` was getting large.
- Exact_in: I was having trouble articulating this one. We need some way of knowing the token type of input; Easiest way was to make the client provide it in a boolean. 
- PDA Derivation: The `state_account` PDA was previously derived using admin's pubkey + `b"state"`. This meant users would need to pass in admin pubkey with their transactions to the pool... which is not ergonomic. It now uses the mints as seeds. 
- Secure Updates: Only the admin whose key is saved in `state_account` can update that state account. This is enforced by requiring a matching signature on any Update transactions. 
- Integer Math: By the end of interview, we agreed the `x_to_y_price` could be f64. However it is safer to avoid float types as different cpu architectures may compute floats differently; leading validators to disagree. I added a scaling factor to the state account and use that in computations. 
- Fees Applied: I put in the logic for applying spread/ comparing their minimum output.


TODO: 
- Get test code working
- PnL tracking:
    - Ideas: Our PnL is affected from the following sources:
        - Price exposure of the liquidity we initially provided
        - Assume our pricing is 100% accurate and up-to-date, so no loss there
        - 
        - Revenue comes from fees. Add state variable for fees_earned, increment it after swap. `earned_x_tokens: u64` and `earned_y_tokens: u64`
    - 
- Add tx expiration timer:
    - Ideas:
        - add an argument `curr_time` to every tx. 
        - add state variable `exp_duration`.
        - in the handler, compare given_time to `Clock::now()` at the very end (after all CPIs), return Err if it exceeds duration. All CPIs are undone if Err is returned. 
        

## Citations
Inspiration was taken from the following sources. 
- https://github.com/solana-developers/program-examples/tree/main/tokens/token-swap
- https://examples.anchor-lang.com/docs/non-custodial-escrow
