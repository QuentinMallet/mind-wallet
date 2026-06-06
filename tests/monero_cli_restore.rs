use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

use mind_wallet::core::{SeedMaterial, derive_monero_wallet_material, mnemonic_from_seed_material};

const RUN_ENV: &str = "MIND_WALLET_RUN_MONERO_CLI";
const CLI_ENV: &str = "MONERO_WALLET_CLI";

fn decode_seed(hex_seed: &str) -> SeedMaterial {
    assert_eq!(hex_seed.len(), 64);
    let mut seed = [0u8; 32];

    for (index, chunk) in hex_seed.as_bytes().chunks(2).enumerate() {
        let hex = std::str::from_utf8(chunk).expect("hex seed should be utf-8");
        seed[index] = u8::from_str_radix(hex, 16).expect("hex seed should decode");
    }

    seed
}

fn official_cli() -> Option<PathBuf> {
    if env::var(RUN_ENV).ok().as_deref() != Some("1") {
        eprintln!("skipping official Monero CLI restore test; set {RUN_ENV}=1 to run it");
        return None;
    }

    Some(
        env::var_os(CLI_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("monero-wallet-cli")),
    )
}

fn wallet_path(test_name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after epoch")
        .as_nanos();
    env::temp_dir()
        .join(format!("mind-wallet-{test_name}-{unique}"))
        .join("wallet")
}

fn remove_wallet_dir(wallet: &Path) {
    if let Some(parent) = wallet.parent() {
        let _ = fs::remove_dir_all(parent);
    }
}

fn create_wallet_dir(wallet: &Path) {
    fs::create_dir_all(
        wallet
            .parent()
            .expect("wallet path should include a parent directory"),
    )
    .expect("wallet directory should be created");
}

fn log_file(wallet: &Path) -> String {
    wallet
        .parent()
        .expect("wallet path should include a parent directory")
        .join("monero-wallet-cli.log")
        .to_str()
        .expect("log path should be utf-8")
        .to_owned()
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "monero-wallet-cli failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_with_input(command: &mut Command, input: &str) -> Output {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("official monero-wallet-cli should run");

    child
        .stdin
        .as_mut()
        .expect("stdin should be piped")
        .write_all(input.as_bytes())
        .expect("restore input should write");

    child
        .wait_with_output()
        .expect("monero-wallet-cli should exit")
}

#[test]
#[ignore = "requires official monero-wallet-cli; run with MIND_WALLET_RUN_MONERO_CLI=1"]
fn official_cli_restores_generated_mnemonic_seed_phrase() {
    let Some(cli) = official_cli() else {
        return;
    };
    let seed = decode_seed("82a13b87b69555ba976601302e2498aed4875185c87b9133bf8d214f16e9eb0b");
    let mnemonic = mnemonic_from_seed_material(seed);
    let wallet = wallet_path("seed-restore");
    create_wallet_dir(&wallet);
    let log_file = log_file(&wallet);

    let output = run_with_input(
        Command::new(cli).args([
            "--offline",
            "--no-dns",
            "--log-file",
            &log_file,
            "--generate-new-wallet",
            wallet.to_str().expect("wallet path should be utf-8"),
            "--restore-deterministic-wallet",
            "--electrum-seed",
            &mnemonic,
            "--password",
            "",
            "--restore-height",
            "0",
            "--command",
            "address",
        ]),
        "\n\n\n",
    );

    remove_wallet_dir(&wallet);
    assert_success(&output);

    let stdout =
        String::from_utf8(output.stdout).expect("monero-wallet-cli stdout should be utf-8");
    assert!(stdout.contains(
        "4BGKFihji4RUj1cygoQjNkDZCRQJ7HvjT82C3bwYkY6zeEP71Ny62nBBy7jVrzojYYKDZfbu5JYoobH7NvdQRfG6MCvjJ59"
    ));
}

#[test]
#[ignore = "requires official monero-wallet-cli; run with MIND_WALLET_RUN_MONERO_CLI=1"]
fn official_cli_restores_generated_key_address_bundle() {
    let Some(cli) = official_cli() else {
        return;
    };
    let seed = decode_seed("3eb8e283b45559d4d2fb6b3a4f52443b420e6da2b38832ea0eb642100c92d600");
    let wallet_material = derive_monero_wallet_material(seed);
    let wallet = wallet_path("key-restore");
    create_wallet_dir(&wallet);
    let log_file = log_file(&wallet);
    let restore_input = format!(
        "{}\n{}\n{}\n0\n",
        wallet_material.primary_address,
        wallet_material.private_spend_key,
        wallet_material.private_view_key
    );

    let output = run_with_input(
        Command::new(cli).args([
            "--offline",
            "--no-dns",
            "--log-file",
            &log_file,
            "--generate-from-keys",
            wallet.to_str().expect("wallet path should be utf-8"),
            "--password",
            "",
            "--restore-height",
            "0",
            "--command",
            "address",
        ]),
        &restore_input,
    );
    remove_wallet_dir(&wallet);
    assert_success(&output);

    let stdout =
        String::from_utf8(output.stdout).expect("monero-wallet-cli stdout should be utf-8");
    assert!(stdout.contains(&wallet_material.primary_address));
}
