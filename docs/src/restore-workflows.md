# Restore Workflows

Use official Monero wallet software to restore and operate the wallet.

## Restore From Mnemonic

Generate the mnemonic:

```sh
mind-wallet --profile v1 --passphrase "alpha bravo charlie delta echo foxtrot golf hotel india juliet kilo lima mike november oscar papa"
```

Restore with official `monero-wallet-cli`:

```sh
monero-wallet-cli --restore-deterministic-wallet
```

When prompted, enter the generated 25-word mnemonic seed phrase. If you use command-line seed flags such as `--electrum-seed`, remember that the seed can be stored in shell history and visible in process listings.

## Restore From Keys

Generate the key/address bundle:

```sh
mind-wallet --profile v1 --passphrase "alpha bravo charlie delta echo foxtrot golf hotel india juliet kilo lima mike november oscar papa" --qr
```

Restore with official `monero-wallet-cli`:

```sh
monero-wallet-cli --generate-from-keys wallet-name
```

When prompted, enter:

- primary address,
- private spend key,
- private view key,
- restore height.

The key/address bundle is also covered by the opt-in official CLI restore test.
