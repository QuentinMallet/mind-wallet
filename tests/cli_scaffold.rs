use std::process::Command;

fn mind_wallet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_mind-wallet"))
}

const TEST_PASSPHRASE: &str = "alpha bravo charlie delta echo foxtrot golf hotel india juliet kilo lima mike november oscar papa";

#[test]
fn help_exposes_scaffold_options() {
    let output = mind_wallet()
        .arg("--help")
        .output()
        .expect("mind-wallet --help should run");

    assert!(output.status.success(), "--help should exit successfully");

    let stdout = String::from_utf8(output.stdout).expect("help output should be utf-8");
    assert!(stdout.contains("Derive Monero wallet material"));
    assert!(stdout.contains("--config"));
    assert!(stdout.contains("--passphrase"));
    assert!(stdout.contains("--profile"));
    assert!(stdout.contains("--qr"));
    assert!(stdout.contains("--verbose"));
}

#[test]
fn default_derivation_outputs_mnemonic_and_warnings() {
    let output = mind_wallet()
        .args(["--passphrase", TEST_PASSPHRASE])
        .output()
        .expect("mind-wallet derivation should run");

    assert!(
        output.status.success(),
        "default derivation should exit successfully"
    );

    let stdout = String::from_utf8(output.stdout).expect("derivation output should be utf-8");
    let words = stdout.split_whitespace().collect::<Vec<_>>();
    assert_eq!(words.len(), 16, "default stdout should be only a mnemonic");

    let stderr = String::from_utf8(output.stderr).expect("warning output should be utf-8");
    assert!(stderr.contains("Profile: v1"));
    assert!(stderr.contains("WARNING"));
    assert!(stderr.contains("terminal output exposes wallet secrets"));
}

#[test]
fn qr_mode_outputs_key_address_bundle_and_terminal_qr() {
    let output = mind_wallet()
        .args(["--passphrase", TEST_PASSPHRASE, "--qr"])
        .output()
        .expect("mind-wallet qr derivation should run");

    assert!(
        output.status.success(),
        "qr derivation should exit successfully"
    );

    let stdout = String::from_utf8(output.stdout).expect("qr output should be utf-8");
    assert!(stdout.contains("Mnemonic seed phrase:"));
    assert!(stdout.contains("Private spend key:"));
    assert!(stdout.contains("Private view key:"));
    assert!(stdout.contains("Primary address:"));
    assert!(stdout.contains("Terminal QR code:"));
    assert!(
        stdout.contains('\u{2580}') || stdout.contains('\u{2584}') || stdout.contains('\u{2588}')
    );

    let stderr = String::from_utf8(output.stderr).expect("warning output should be utf-8");
    assert!(stderr.contains("Profile: v1"));
    assert!(stderr.contains("QR output exposes private keys"));
}
