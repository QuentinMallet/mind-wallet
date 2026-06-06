# Introduction

`mind-wallet` deterministically derives Monero wallet restore material from a memorized passphrase.

It outputs:

- a Monero-compatible 25-word mnemonic seed phrase,
- a private spend key,
- a private view key,
- a primary mainnet address,
- an optional terminal QR code containing the key/address restore bundle.

This tool is intentionally small. It does not connect to a daemon, scan a chain, store wallet files, or manage balances. Use the official Monero wallet tools for restoration, synchronization, transactions, and account operations.

## Non-Goals

- This is not a password manager.
- This is not a replacement for `monero-wallet-cli`.
- This is not a way to make weak passphrases safe.
- This does not protect secrets once they are printed to a terminal, shell history, scrollback, logs, screenshots, or QR scanners.
