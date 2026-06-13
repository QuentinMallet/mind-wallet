// The mind-wallet CLI is a host-only binary; the wasm32 build produces
// only the `cdylib` library used by the portfolio demo. We still keep a
// stub `main` here so `cargo build --tests --target wasm32-unknown-unknown`
// (which `wasm-pack test` invokes) does not try to link an unresolvable
// `mind_wallet::application` import.

#[cfg(not(target_family = "wasm"))]
fn main() {
    use abscissa_core::application;
    use mind_wallet::application::APPLICATION;
    application::boot(&APPLICATION);
}

#[cfg(target_family = "wasm")]
fn main() {}
