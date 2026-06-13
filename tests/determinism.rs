//! Determinism oracle for the mind-wallet derivation pipeline.
//!
//! Derivation MUST be a pure function of the passphrase and the profile:
//! same input → byte-identical mnemonic, address, and key bundle, every
//! invocation, on every target. This invariant is load-bearing for the
//! WASM portfolio demo (parity with the CLI) and for any future
//! deterministic restore workflow.
//!
//! This test catches any future regression that injects a non-deterministic
//! source — `thread_rng`, `OsRng`, system time, an environment lookup — at
//! any layer of the seed → keypair → address chain.

use mind_wallet::core::{
    Profile, derive_monero_wallet_material, derive_seed_material,
};

const FIXED_PASSPHRASE: &str =
    "alpha bravo charlie delta echo foxtrot golf hotel india juliet kilo lima mike november oscar papa";
const ITERATIONS: usize = 10;

#[test]
fn derivation_is_byte_identical_across_repeats() {
    let profile = Profile::v1();
    let seed_0 = derive_seed_material(FIXED_PASSPHRASE, profile)
        .expect("argon2 succeeds on iteration 0");
    let wallet_0 = derive_monero_wallet_material(seed_0);

    for i in 1..ITERATIONS {
        let seed_i = derive_seed_material(FIXED_PASSPHRASE, profile)
            .expect("argon2 succeeds on every iteration");
        assert_eq!(
            seed_i, seed_0,
            "seed material diverged at iteration {i}",
        );

        let wallet_i = derive_monero_wallet_material(seed_i);
        assert_eq!(
            wallet_i.mnemonic_seed_phrase, wallet_0.mnemonic_seed_phrase,
            "mnemonic diverged at iteration {i}",
        );
        assert_eq!(
            wallet_i.private_spend_key, wallet_0.private_spend_key,
            "private spend key diverged at iteration {i}",
        );
        assert_eq!(
            wallet_i.private_view_key, wallet_0.private_view_key,
            "private view key diverged at iteration {i}",
        );
        assert_eq!(
            wallet_i.primary_address, wallet_0.primary_address,
            "primary address diverged at iteration {i}",
        );
    }
}
