//! WebAssembly bindings for the portfolio demo at https://mind-wallet.mstratsec.biz/.
//!
//! This module is gated to `cfg(target_family = "wasm")` so the native CLI
//! build sees only an empty module — no host-only deps leak in either
//! direction. Per Principle #1, the CLI binary's symbol set must be
//! preserved (see `tests/determinism.rs` and the flake `nm -D` gate).
//!
//! ## Security invariants
//!
//! - `DerivationResult::seed_preview_hex` exposes the first N bytes of the
//!   20-byte seed where N ≤ 8 (40 bits). The full seed never crosses the
//!   JS boundary except inside `mnemonic` (BIP39-style word list) or
//!   `bundle` (CLI-format restore string). Do NOT add a `seed_full_hex`
//!   field; do NOT widen this preview past 8 bytes.
//! - All derivation happens client-side. No fetch/XHR/WebSocket from this
//!   module — see the zero-network grep gate in
//!   `.github/workflows/deploy-pages.yml`.
//! - Derivation must remain deterministic per the invariant declared in
//!   `src/core/monero/mod.rs` — `getrandom`'s `js` feature is wired only
//!   to satisfy the link contract for `curve25519-dalek`; nothing in the
//!   seed → keypair → address chain consumes randomness.

use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::core::{
    Profile, derive_monero_wallet_material, derive_seed_material,
};

/// Hand-rolled TypeScript declaration for `DerivationResult` so the `.d.ts`
/// surface enumerates every field even though the Rust → JS marshalling
/// goes through `serde_wasm_bindgen::to_value` (which strips Rust type info
/// down to `any`). Keep this in sync with the struct below.
#[wasm_bindgen(typescript_custom_section)]
const TS_DERIVATION_RESULT: &'static str = r#"
export interface DerivationResult {
  /** BIP39-style mnemonic seed phrase (16 derived words + 1 checksum word). */
  mnemonic: string;
  /** CLI-format restore bundle: spend key | view key | primary address. */
  bundle: string;
  /** Monero primary address (Base58, mainnet). */
  address: string;
  /** First 8 bytes of the Argon2id output (16 hex chars). Preview only. */
  kdf_preview_hex: string;
  /** First 8 bytes of the 20-byte seed (16 hex chars). Preview only. */
  seed_preview_hex: string;
  /** Public spend key, hex-encoded (64 chars). */
  spend_pub_hex: string;
  /** Public view key, hex-encoded (64 chars). */
  view_pub_hex: string;
  /** Human-readable profile summary string (mirrors the CLI banner). */
  profile_summary: string;
  /** Argon2id KDF wall time in ms (measured via performance.now()). */
  kdf_ms: number;
  /** Seed-material allocation/copy wall time in ms. */
  seed_ms: number;
  /** Keypair derivation wall time in ms. */
  keypair_ms: number;
  /** Address-formatting wall time in ms. */
  address_ms: number;
}

/** Error returned by `derive()` on failure. */
export interface DeriveError {
  code: "passphrase_word_count" | "profile_unknown" | "derivation_failed" | "qr_failed";
  message: string;
}
"#;

const REQUIRED_PASSPHRASE_WORDS: usize = 16;
const SEED_PREVIEW_BYTES: usize = 8;

/// First N hex characters of `bytes` (N = 2 * SEED_PREVIEW_BYTES = 16 chars).
fn hex_preview(bytes: &[u8]) -> String {
    let take = SEED_PREVIEW_BYTES.min(bytes.len());
    let mut out = String::with_capacity(take * 2);
    for b in &bytes[..take] {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

fn full_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

/// Compute the ed25519 public key bytes for a Monero private-key scalar.
///
/// Monero's keys are ed25519 in canonical form: `pubkey = scalar * B` where
/// `B` is the ed25519 basepoint. We mirror the derivation `monero` does
/// internally, but in a way that keeps the WASM-facing surface free of the
/// `monero::util::key::PublicKey` constructor (whose name/signature varies
/// across `monero` crate versions).
fn pub_key_hex_from_scalar_bytes(scalar_bytes: &[u8; 32]) -> String {
    use curve25519_dalek::constants::ED25519_BASEPOINT_TABLE;
    use curve25519_dalek::scalar::Scalar;

    let scalar = Scalar::from_bytes_mod_order(*scalar_bytes);
    let point = &scalar * ED25519_BASEPOINT_TABLE;
    full_hex(&point.compress().to_bytes())
}

fn profile_summary(profile: Profile) -> String {
    format!(
        "Profile: v1 (Argon2id m={} KiB, t={}, p={}, domain={})",
        profile.memory_cost_kib(),
        profile.time_cost(),
        profile.parallelism(),
        profile.domain()
    )
}

fn key_address_bundle(wallet: &crate::core::MoneroWalletMaterial) -> String {
    format!(
        "Private spend key: {}\nPrivate view key: {}\nPrimary address: {}",
        wallet.private_spend_key, wallet.private_view_key, wallet.primary_address
    )
}

fn selected_profile(profile: &str) -> Option<Profile> {
    match profile {
        "v1" => Some(Profile::v1()),
        _ => None,
    }
}

fn word_count(passphrase: &str) -> usize {
    passphrase.split_whitespace().count()
}

/// Serializable structured derivation output. All 12 fields are part of the
/// public TypeScript surface — `pkg/mind_wallet.d.ts` must surface every
/// field. Bumping or renaming any of these is a JS breakage.
#[derive(Serialize)]
pub struct DerivationResult {
    pub mnemonic: String,
    pub bundle: String,
    pub address: String,
    /// First 8 bytes of the Argon2id output hex (16 chars). Preview only.
    pub kdf_preview_hex: String,
    /// First 8 bytes of the 20-byte seed material hex (16 chars). Preview only.
    pub seed_preview_hex: String,
    pub spend_pub_hex: String,
    pub view_pub_hex: String,
    pub profile_summary: String,
    /// Per-stage timings measured via `performance.now()` deltas. The
    /// walkthrough cards render these so the page never lies about how long
    /// the math actually took (Architect A6 / Critic N8).
    pub kdf_ms: f64,
    pub seed_ms: f64,
    pub keypair_ms: f64,
    pub address_ms: f64,
}

#[derive(Serialize)]
struct DeriveError {
    code: &'static str,
    message: String,
}

fn js_err(code: &'static str, message: impl Into<String>) -> JsValue {
    serde_wasm_bindgen::to_value(&DeriveError {
        code,
        message: message.into(),
    })
    .unwrap_or_else(|_| JsValue::from_str(code))
}

fn now_ms() -> f64 {
    js_sys::Date::now()
}

/// Tiny ping export — used by the W1b spike acceptance test.
#[wasm_bindgen]
pub fn ping() -> String {
    "pong".to_string()
}

/// Word count for a candidate passphrase. The page calls this synchronously
/// before invoking `derive` so the visitor sees inline validation feedback
/// without paying the Argon2id cost.
#[wasm_bindgen]
pub fn validate_passphrase(passphrase: &str) -> bool {
    word_count(passphrase) == REQUIRED_PASSPHRASE_WORDS
}

/// Returns the list of supported derivation profile identifiers. The page
/// populates its `<select>` from this list so adding a profile here is the
/// only change needed when v2 lands.
#[wasm_bindgen]
pub fn supported_profiles() -> Vec<JsValue> {
    vec![JsValue::from_str("v1")]
}

/// Renders an SVG QR-code for the given payload. The page injects the
/// returned markup directly into a `.qr` `<div>` (no `<img>` round-trip
/// through a blob URL — keeps the zero-network promise intact).
#[wasm_bindgen]
pub fn render_qr_svg(payload: &str) -> Result<String, JsValue> {
    use qrcode::QrCode;
    use qrcode::render::svg;

    let code = QrCode::new(payload.as_bytes())
        .map_err(|e| js_err("qr_failed", format!("{e}")))?;
    let svg = code
        .render::<svg::Color>()
        .min_dimensions(256, 256)
        .quiet_zone(true)
        .dark_color(svg::Color("#F4ECDA"))
        .light_color(svg::Color("#1A1A1A"))
        .build();
    Ok(svg)
}

/// Run the full derivation pipeline and return a structured result with
/// per-stage timings.
///
/// Errors are returned as `{ code, message }` JS objects. Defined codes:
/// - `passphrase_word_count` — input does not contain exactly 16 words.
/// - `profile_unknown` — profile identifier not in `supported_profiles()`.
/// - `derivation_failed` — Argon2id reported an error.
#[wasm_bindgen]
pub fn derive(passphrase: &str, profile: &str) -> Result<JsValue, JsValue> {
    if !validate_passphrase(passphrase) {
        return Err(js_err(
            "passphrase_word_count",
            format!(
                "expected {REQUIRED_PASSPHRASE_WORDS} space-separated words, got {}",
                word_count(passphrase)
            ),
        ));
    }

    let Some(profile_value) = selected_profile(profile) else {
        return Err(js_err(
            "profile_unknown",
            format!("unsupported profile '{profile}'; supported: v1"),
        ));
    };

    // Argon2id is the dominant cost; everything else in the chain takes
    // sub-millisecond. We measure the two stages that do real work
    // (`kdf_ms`, `keypair_ms`) and honestly report 0 for the two that
    // are nominal (`seed_ms` is the same bytes as the KDF output, no
    // extra work; `address_ms` is already accounted for in
    // `derive_monero_wallet_material`). The walkthrough cards display
    // these literally so a visitor reading "0 ms" learns something true
    // about the pipeline rather than seeing a synthesised animation
    // delay (Architect A6 / Critic N8 — honest timings).
    let t_kdf_start = now_ms();
    let seed = derive_seed_material(passphrase, profile_value).map_err(|e| {
        js_err("derivation_failed", format!("argon2 failed: {e}"))
    })?;
    let kdf_ms = now_ms() - t_kdf_start;

    let t_keys_start = now_ms();
    let wallet = derive_monero_wallet_material(seed);
    let keypair_ms = now_ms() - t_keys_start;

    // Pubkeys for the walkthrough cards. `derive_monero_wallet_material`
    // already computes these internally to build the address but does
    // not expose them, so we re-derive (basepoint × scalar) from the
    // private spend scalar and the Keccak-derived view scalar — both
    // deterministic, both byte-identical to what the address encodes.
    use curve25519_dalek::scalar::Scalar;
    use monero::{Hash, PrivateKey};
    let mut spend_scalar = [0u8; 32];
    spend_scalar[..20].copy_from_slice(&seed);
    let spend_pub_hex = pub_key_hex_from_scalar_bytes(&spend_scalar);
    let private_spend = PrivateKey::from_scalar(Scalar::from_bytes_mod_order(spend_scalar));
    let private_view = Hash::hash_to_scalar(private_spend.to_bytes());
    let view_pub_hex = pub_key_hex_from_scalar_bytes(&private_view.to_bytes());

    let result = DerivationResult {
        mnemonic: wallet.mnemonic_seed_phrase.clone(),
        bundle: key_address_bundle(&wallet),
        address: wallet.primary_address.clone(),
        kdf_preview_hex: hex_preview(&seed),
        seed_preview_hex: hex_preview(&seed),
        spend_pub_hex,
        view_pub_hex,
        profile_summary: profile_summary(profile_value),
        kdf_ms,
        seed_ms: 0.0,
        keypair_ms,
        address_ms: 0.0,
    };

    serde_wasm_bindgen::to_value(&result).map_err(|e| {
        js_err(
            "derivation_failed",
            format!("serialize: {e}"),
        )
    })
}

#[wasm_bindgen(start)]
pub fn __wasm_start() {
    console_error_panic_hook::set_once();
}
