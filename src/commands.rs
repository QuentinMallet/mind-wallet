use crate::{
    config::AppConfig,
    core::{Profile, derive_monero_wallet_material, derive_seed_material},
};
use abscissa_core::{Command, Configurable, Runnable};
use clap::Parser;
use qrcode::{QrCode, render::unicode};
use std::path::PathBuf;


#[derive(Command, Debug, Parser)]
#[command(author, about, version)]
pub struct EntryPoint {
    /// Enable verbose diagnostic output.
    #[arg(short, long)]
    pub verbose: bool,

    /// Load configuration from the specified TOML file.
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Passphrase to deterministically derive the Monero wallet material from.
    #[arg(long, value_name = "PASSPHRASE")]
    pub passphrase: String,

    /// Deterministic derivation profile to use.
    #[arg(long, default_value = "v1", value_name = "ID")]
    pub profile: String,

    /// Print a terminal QR code for the private key/address restore bundle.
    #[arg(long)]
    pub qr: bool,
}

impl Configurable<AppConfig> for EntryPoint {
    fn config_path(&self) -> Option<PathBuf> {
        self.config.clone()
    }
}

impl Runnable for EntryPoint {
    fn run(&self) {
        if self.verbose {
            eprintln!("mind-wallet derivation running in verbose mode");
        }

        let profile = match selected_profile(&self.profile) {
            Some(profile) => profile,
            None => {
                eprintln!(
                    "unsupported profile '{}'; supported profiles: v1",
                    self.profile
                );
                std::process::exit(2);
            }
        };

        eprintln!("{}", profile_summary(profile));
        eprintln!("WARNING: --passphrase may be visible in shell history and process listings");
        eprintln!("WARNING: terminal output exposes wallet secrets");

        if self.qr {
            eprintln!("WARNING: QR output exposes private keys");
        }

        let seed = match derive_seed_material(&self.passphrase, profile) {
            Ok(seed) => seed,
            Err(error) => {
                eprintln!("failed to derive seed material: {error}");
                std::process::exit(1);
            }
        };
        let wallet = derive_monero_wallet_material(seed);

        if self.qr {
            let bundle = key_address_bundle(&wallet);
            let qr = match QrCode::new(bundle.as_bytes()) {
                Ok(code) => code.render::<unicode::Dense1x2>().quiet_zone(false).build(),
                Err(error) => {
                    eprintln!("failed to generate QR code: {error}");
                    std::process::exit(1);
                }
            };

            println!("Mnemonic seed phrase:");
            println!("{}", wallet.mnemonic_seed_phrase);
            println!();
            println!("{bundle}");
            println!();
            println!("Terminal QR code:");
            println!("{qr}");
        } else {
            println!("{}", wallet.mnemonic_seed_phrase);
        }
    }
}


fn selected_profile(profile: &str) -> Option<Profile> {
    match profile {
        "v1" => Some(Profile::v1()),
        _ => None,
    }
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
