# Usage

Derive the default Monero mnemonic seed phrase:

```sh
mind-wallet --passphrase "alpha bravo charlie delta echo foxtrot golf hotel india juliet kilo lima mike november oscar papa"
```

The passphrase must contain exactly 16 whitespace-separated words. Default stdout is only the 25-word mnemonic phrase so it can be copied into an official Monero restore workflow. Profile metadata and warnings are printed to stderr.

Print the key/address restore bundle and terminal QR code:

```sh
mind-wallet --passphrase "alpha bravo charlie delta echo foxtrot golf hotel india juliet kilo lima mike november oscar papa" --qr
```

The QR output encodes:

- private spend key,
- private view key,
- primary address.

Select the released profile explicitly:

```sh
mind-wallet --profile v1 --passphrase "alpha bravo charlie delta echo foxtrot golf hotel india juliet kilo lima mike november oscar papa"
```

## Output Warning

`--passphrase` can be visible in shell history and process listings. Terminal output can be captured by scrollback, logs, screenshots, remote sessions, or clipboard managers. QR output exposes private keys to the display, camera, and any QR reader used.

Run only in an environment where terminal output is treated as secret material.
