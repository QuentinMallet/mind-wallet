use mind_wallet::core::{Profile, ProfileId, derive_seed_material};

fn hex_encode(bytes: impl AsRef<[u8]>) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let bytes = bytes.as_ref();
    let mut encoded = String::with_capacity(bytes.len() * 2);

    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }

    encoded
}

#[test]
fn v1_profile_metadata_is_stable() {
    let profile = Profile::v1();

    assert_eq!(profile.id(), ProfileId::V1);
    assert_eq!(profile.domain(), "mind-wallet-monero-v1");
    assert_eq!(profile.memory_cost_kib(), 19 * 1024);
    assert_eq!(profile.time_cost(), 2);
    assert_eq!(profile.parallelism(), 1);
    assert_eq!(profile.output_len(), 20);
}

#[test]
fn same_passphrase_and_profile_derive_same_seed_material() {
    let profile = Profile::v1();

    let first = derive_seed_material("correct horse battery staple", profile)
        .expect("first derivation should succeed");
    let second = derive_seed_material("correct horse battery staple", profile)
        .expect("second derivation should succeed");

    assert_eq!(first, second);
}

#[test]
fn different_passphrases_derive_different_seed_material() {
    let profile = Profile::v1();

    let first =
        derive_seed_material("correct horse battery staple", profile).expect("first derivation");
    let second =
        derive_seed_material("correct horse battery staples", profile).expect("second derivation");

    assert_ne!(first, second);
}

#[test]
fn v1_golden_vector_is_reproducible() {
    let seed = derive_seed_material("mind-wallet test vector", Profile::v1())
        .expect("golden vector derivation should succeed");

    assert_eq!(
        hex_encode(seed),
        "2aab25d06cd39a33bcd36f7047e596966b59f9de"
    );
}
