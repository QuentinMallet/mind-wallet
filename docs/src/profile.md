# Derivation Profile

Profiles are immutable compatibility contracts. Once a profile is released, changing its parameters would derive a different wallet from the same passphrase.

## `v1`

Profile metadata:

- domain: `mind-wallet-monero-v1`
- KDF: Argon2id
- Argon2 version: 0x13
- memory: 19 MiB
- time cost: 2
- parallelism: 1
- output length: 32 bytes
- network: Monero mainnet

The 32-byte KDF output is reduced to a Monero private spend scalar. The private view key is derived with Monero's hash-to-scalar convention from the private spend key. The mnemonic phrase uses the Monero English 25-word seed encoding.

## Compatibility Rule

Never change an existing profile. Add a new profile id for any future change to KDF parameters, domain string, mnemonic encoding, network, or key derivation behavior.
