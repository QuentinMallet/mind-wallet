//! WASM-target parity oracle. Runs only under `wasm-pack test`.
//!
//! Confirms that the `wasm::derive()` JS-facing export returns the same
//! mnemonic / spend / view / address as the locked native fixtures. This
//! exercises the wasm-bindgen + serde_wasm_bindgen marshalling layer, so
//! any future regression in field naming or encoding fails here before
//! reaching the browser.

#![cfg(target_arch = "wasm32")]

mod common;

use common::FIXTURES;
use serde::Deserialize;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

// No `run_in_browser` configure — node is the canonical CI runner for
// these parity tests (`wasm-pack test --node`). Browser runs still work
// (`wasm-pack test --headless --firefox`) but are documented as developer
// machine smoke, not CI gates.

#[derive(Deserialize)]
struct DerivationResultJs {
    mnemonic: String,
    bundle: String,
    address: String,
    kdf_preview_hex: String,
    seed_preview_hex: String,
    spend_pub_hex: String,
    view_pub_hex: String,
}

#[wasm_bindgen_test]
fn wasm_derive_matches_locked_fixtures() {
    for f in FIXTURES {
        let value: JsValue = mind_wallet::wasm::derive(f.passphrase, "v1")
            .expect("derive() succeeds on every fixture");
        let result: DerivationResultJs =
            serde_wasm_bindgen::from_value(value).expect("DerivationResult deserializes");

        assert_eq!(result.mnemonic, f.mnemonic);
        assert_eq!(result.address, f.address);
        // The bundle is the concatenation the CLI builds.
        let expected_bundle = format!(
            "Private spend key: {}\nPrivate view key: {}\nPrimary address: {}",
            f.spend, f.view, f.address
        );
        assert_eq!(result.bundle, expected_bundle);

        // Preview invariant: ≤16 hex chars (≤8 bytes).
        assert!(
            result.kdf_preview_hex.len() <= 16,
            "kdf preview must be <=16 hex chars, got {}",
            result.kdf_preview_hex.len()
        );
        assert!(
            result.seed_preview_hex.len() <= 16,
            "seed preview must be <=16 hex chars, got {}",
            result.seed_preview_hex.len()
        );

        // Public keys hex-encoded (32 bytes → 64 chars).
        assert_eq!(result.spend_pub_hex.len(), 64);
        assert_eq!(result.view_pub_hex.len(), 64);
    }
}

#[wasm_bindgen_test]
fn wasm_validate_passphrase_enforces_16_words() {
    assert!(mind_wallet::wasm::validate_passphrase(
        "alpha bravo charlie delta echo foxtrot golf hotel india juliet kilo lima mike november oscar papa"
    ));
    assert!(!mind_wallet::wasm::validate_passphrase("too few words"));
    assert!(!mind_wallet::wasm::validate_passphrase(""));
}

#[wasm_bindgen_test]
fn wasm_derive_rejects_bad_passphrase_with_typed_code() {
    let err: JsValue = mind_wallet::wasm::derive("short input", "v1")
        .err()
        .expect("must error on short input");

    #[derive(Deserialize)]
    struct DeriveErr { code: String, message: String }
    let parsed: DeriveErr = serde_wasm_bindgen::from_value(err).unwrap();
    assert_eq!(parsed.code, "passphrase_word_count");
    assert!(!parsed.message.is_empty());
}
