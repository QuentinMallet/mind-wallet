use mind_wallet::core::{SeedMaterial, derive_monero_wallet_material, mnemonic_from_seed_material};

fn decode_seed(hex_seed: &str) -> SeedMaterial {
    assert_eq!(hex_seed.len(), 64);
    let mut seed = [0u8; 32];

    for (index, chunk) in hex_seed.as_bytes().chunks(2).enumerate() {
        let hex = std::str::from_utf8(chunk).expect("hex seed should be utf-8");
        seed[index] = u8::from_str_radix(hex, 16).expect("hex seed should decode");
    }

    seed
}

#[test]
fn mnemonic_from_seed_matches_original_english_vector() {
    let seed = decode_seed("82a13b87b69555ba976601302e2498aed4875185c87b9133bf8d214f16e9eb0b");

    assert_eq!(
        mnemonic_from_seed_material(seed),
        "reruns today hookup itself thorn nirvana symptoms jukebox patio unquoted sushi long diode digit rewind hacksaw obvious soothe nightly return agile hobby algebra awesome nirvana"
    );
}

#[test]
fn wallet_material_from_seed_matches_mainnet_vector() {
    let seed = decode_seed("3eb8e283b45559d4d2fb6b3a4f52443b420e6da2b38832ea0eb642100c92d600");

    let wallet = derive_monero_wallet_material(seed);

    assert_eq!(
        wallet.private_spend_key,
        "3eb8e283b45559d4d2fb6b3a4f52443b420e6da2b38832ea0eb642100c92d600"
    );
    assert_eq!(
        wallet.private_view_key,
        "5177c436f032666c572df97ab591cc6ac2da96ab6818a2f38d72b430aebbdc0a"
    );
    assert_eq!(
        wallet.primary_address,
        "48fRSJiQSp3Da61k8NSR5J9ibWMBkrJHL3hGDxSaZJvsfK7jpigPWyyGy5jqs8MSgeCBQb1HR4NDS84goPRaLV2xTungkh5"
    );
}

#[test]
fn wallet_material_includes_mnemonic_phrase() {
    let seed = decode_seed("31e28ef4feca46915bdbf7b192af866e154cb7dbc704e9a39b6ce24ac89c1102");

    let wallet = derive_monero_wallet_material(seed);

    assert_eq!(
        wallet.mnemonic_seed_phrase,
        "cafe aided wounded lumber hounded water yoyo gasp aerial merger ungainly gaze ruby yacht tell playful smash issued sifting whole erase anxiety dash deity sifting"
    );
}
