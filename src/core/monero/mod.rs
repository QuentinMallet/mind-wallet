//! INVARIANT: derivation MUST be purely deterministic from the 20-byte
//! seed. No `thread_rng`, no `OsRng`, no system-time inputs anywhere in
//! the seed → keypair → address chain. The mind-wallet WASM portfolio
//! demo relies on this for native↔wasm parity and for the determinism
//! oracle in `tests/determinism.rs`.

use crc32fast::Hasher;
use curve25519_dalek::scalar::Scalar;
use monero::{Address, Hash, KeyPair, Network, PrivateKey};

use crate::core::SeedMaterial;

const ENGLISH_WORDS: &str = include_str!("english.txt");
const WORD_COUNT: usize = 1626;
const CHECKSUM_PREFIX_LEN: usize = 3;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MoneroWalletMaterial {
    pub mnemonic_seed_phrase: String,
    pub private_spend_key: String,
    pub private_view_key: String,
    pub primary_address: String,
}

pub fn derive_monero_wallet_material(seed: SeedMaterial) -> MoneroWalletMaterial {
    let private_spend = private_spend_key_from_seed(seed);
    let private_view = private_view_key_from_spend(private_spend);
    let keys = KeyPair {
        view: private_view,
        spend: private_spend,
    };
    let primary_address = Address::from_keypair(Network::Mainnet, &keys);

    MoneroWalletMaterial {
        mnemonic_seed_phrase: mnemonic_from_seed_material(seed),
        private_spend_key: private_spend.to_string(),
        private_view_key: private_view.to_string(),
        primary_address: primary_address.to_string(),
    }
}

pub fn mnemonic_from_seed_material(seed: SeedMaterial) -> String {
    let words = english_words();
    let mut phrase_words = Vec::with_capacity(16);

    for chunk in seed.chunks_exact(4) {
        let index = u32::from_le_bytes(chunk.try_into().expect("chunks_exact yields 4 bytes"));

        let word_1 = index as usize % WORD_COUNT;
        let word_2 = (index as usize / WORD_COUNT + word_1) % WORD_COUNT;
        let word_3 = ((index as usize / WORD_COUNT / WORD_COUNT) + word_2) % WORD_COUNT;

        phrase_words.push(words[word_1]);
        phrase_words.push(words[word_2]);
        phrase_words.push(words[word_3]);
    }

    let checksum_index = checksum_index(&phrase_words);
    phrase_words.push(phrase_words[checksum_index]);
    phrase_words.join(" ")
}

fn private_spend_key_from_seed(seed: SeedMaterial) -> PrivateKey {
    let mut padded = [0u8; 32];
    padded[..20].copy_from_slice(&seed);
    PrivateKey::from_scalar(Scalar::from_bytes_mod_order(padded))
}

fn private_view_key_from_spend(private_spend: PrivateKey) -> PrivateKey {
    Hash::hash_to_scalar(private_spend.to_bytes())
}

fn english_words() -> Vec<&'static str> {
    let words = ENGLISH_WORDS.lines().collect::<Vec<_>>();
    assert_eq!(words.len(), WORD_COUNT);
    words
}

fn checksum_index(words: &[&str]) -> usize {
    let mut hasher = Hasher::new();

    for word in words {
        hasher.update(&word.as_bytes()[..CHECKSUM_PREFIX_LEN]);
    }

    hasher.finalize() as usize % words.len()
}
