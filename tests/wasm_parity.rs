//! Native parity oracle.
//!
//! Asserts that the library's `derive_seed_material` +
//! `derive_monero_wallet_material` pipeline matches the locked
//! `common::FIXTURES` table byte-for-byte. The wasm32 mirror at
//! `tests/wasm_parity_wasm32.rs` runs the same fixtures through the
//! `wasm::derive` export — if both pass, native ↔ WASM derivation parity
//! is established.

mod common;

use common::FIXTURES;
use mind_wallet::core::{
    Profile, derive_monero_wallet_material, derive_seed_material,
};

#[test]
fn cli_native_matches_locked_fixtures() {
    let profile = Profile::v1();

    for f in FIXTURES {
        let seed = derive_seed_material(f.passphrase, profile)
            .expect("argon2 succeeds on every fixture");
        let wallet = derive_monero_wallet_material(seed);

        assert_eq!(
            wallet.mnemonic_seed_phrase, f.mnemonic,
            "mnemonic mismatch for passphrase {:?}",
            f.passphrase
        );
        assert_eq!(
            wallet.private_spend_key, f.spend,
            "spend key mismatch for passphrase {:?}",
            f.passphrase
        );
        assert_eq!(
            wallet.private_view_key, f.view,
            "view key mismatch for passphrase {:?}",
            f.passphrase
        );
        assert_eq!(
            wallet.primary_address, f.address,
            "address mismatch for passphrase {:?}",
            f.passphrase
        );
    }
}
