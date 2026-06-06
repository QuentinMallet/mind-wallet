use super::Profile;
use argon2::{Algorithm, Argon2, Version};

pub type SeedMaterial = [u8; 32];

pub fn derive_seed_material(
    passphrase: impl AsRef<[u8]>,
    profile: Profile,
) -> argon2::Result<SeedMaterial> {
    let params = profile.argon2_params()?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut seed = [0u8; 32];

    argon2.hash_password_into(passphrase.as_ref(), profile.domain().as_bytes(), &mut seed)?;

    Ok(seed)
}
