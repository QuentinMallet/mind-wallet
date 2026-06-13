# mind-wallet

Derive Monero wallet material (mnemonic seed phrase, restore bundle,
address) deterministically from a memorized 16-word passphrase. The CLI
is the canonical tool. A WebAssembly portfolio demo runs the same
derivation in the browser at <https://mind-wallet.mstratsec.biz/>.

## Quick start

```bash
nix develop
cargo run --release -- --passphrase "alpha bravo charlie delta echo foxtrot golf hotel india juliet kilo lima mike november oscar papa" --qr
```

For real wallets, run from an air-gapped machine. The browser demo is
explicitly a showcase, not a recommended path for funds you actually
hold.

## Browser demo

<https://mind-wallet.mstratsec.biz/> serves the same Argon2id → ed25519
→ Monero address pipeline compiled to WebAssembly. Source under `web/`;
deploy workflow under `.github/workflows/deploy-pages.yml`. Brand styling
is loaded live from <https://mstratsec.biz/tokens.css>; supply-chain
takedown runbook in [`web/README.md`](./web/README.md#supply-chain-takedown).

## Documentation

`docs/` is an mdBook. Render with:

```bash
nix build .#doc
xdg-open result/index.html
```

## Support the dev

Monero donation:

```
8Aw2c14zTgP7LPwfjwZGvbbCh87cprqiZb82h4cJiPhb9rv42MxbKazgfXXW69vrfUMYGdRhThC7JgyeAjSDPJ9CDU16vLG
```
