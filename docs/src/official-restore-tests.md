# Official Restore Tests

The default test suite does not require Monero binaries. Official restore compatibility is covered by ignored integration tests in `tests/monero_cli_restore.rs`.

Prerequisites:

- `monero-wallet-cli` from the official Monero CLI distribution or a package that provides the official CLI.
- `MIND_WALLET_RUN_MONERO_CLI=1` to opt in.
- Optional `MONERO_WALLET_CLI=/path/to/monero-wallet-cli` when the binary is not on `PATH`.

Run with Nix:

```sh
MONERO_CLI="$(nix build --no-link --print-out-paths nixpkgs#monero-cli)/bin/monero-wallet-cli"
nix develop -c env MIND_WALLET_RUN_MONERO_CLI=1 MONERO_WALLET_CLI="$MONERO_CLI" cargo test --test monero_cli_restore -- --ignored --nocapture
```

The tests run `monero-wallet-cli` in `--offline --no-dns` mode, create temporary wallet files under the system temp directory, and delete those files after each restore.

Covered restore paths:

- `--restore-deterministic-wallet --electrum-seed` for the generated 25-word mnemonic seed phrase.
- `--generate-from-keys` for the generated primary address, private spend key, and private view key bundle.
