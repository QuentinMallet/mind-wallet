//! Locked parity fixtures shared between the native and the wasm32
//! parity oracles. Each entry pairs a passphrase with the canonical
//! `mind-wallet` CLI output (profile = v1).
//!
//! To regenerate after an intentional algorithm change, run:
//!
//! ```text
//! cargo run --example dump_fixtures
//! ```
//!
//! (or call `mind_wallet::core::derive_*` directly from a scratch binary)
//! and paste the new mnemonic/spend/view/address strings here. Any
//! unintentional algorithm change should fail the parity test instead of
//! being papered over.

#![allow(dead_code)]

pub struct Fixture {
    pub passphrase: &'static str,
    pub mnemonic: &'static str,
    pub spend: &'static str,
    pub view: &'static str,
    pub address: &'static str,
}

pub const FIXTURES: &[Fixture] = &[
    Fixture {
        passphrase: "alpha bravo charlie delta echo foxtrot golf hotel india juliet kilo lima mike november oscar papa",
        mnemonic: "neutral tonic suddenly shackles fishing superior inroads sipped misery western pairing worry jester orange arbitrary misery",
        spend: "c530adf3ae025986dc9c5cc2c8d45a5807980471000000000000000000000000",
        view: "43d669b545be0268c8706a7fb5f3c96ce9ff4025d6f59cf5557364383b504401",
        address: "4A8crL3BQiQCuWPnPEybXsCNjytaBkHD2EJxrh31Q5dehykSNoQZJG6V8iGHByfj4R5BDzSWwRv9dHmCjza9FiL6RnPd1iu",
    },
    Fixture {
        passphrase: "uno dos tres cuatro cinco seis siete ocho nueve diez once doce trece catorce quince dieciseis",
        mnemonic: "turnip obnoxious furnished sovereign unknown awakened justice paddles intended terminal upload plus deodorant sincerely boyfriend intended",
        spend: "fe6b7db8d98ad12e4a0a1cc9e30d00c290bbf858000000000000000000000000",
        view: "33b0c1ac5a0032fbc905d3dd9944eecfb4521657fab189e42f9ef50639f9bf0f",
        address: "48Vni3wmohUW9WDczYN7ntbTyQFvvpCcr45g9VuEUbHBKniPkNmcSuUVCB6CHeuGcnfcszfUqUkwYaEiwvpHVjWQBo5djAN",
    },
];
