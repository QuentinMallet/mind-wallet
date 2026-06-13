// mind-wallet portfolio demo — hand-written ES module.
//
// ZERO-NETWORK PROMISE: this file MUST NOT make outbound network calls.
// See the grep gate in .github/workflows/deploy-pages.yml — it scans
// THIS file (not pkg/, whose wasm-bindgen glue is allowed to load the
// .wasm binary on init). If you need to call out to the network from
// this page, you are diverging from the spec — see
// `.omc/specs/deep-interview-mind-wallet-wasm-portfolio.md` Non-Goals.

import init, {
  derive,
  supported_profiles,
  validate_passphrase,
  render_qr_svg,
} from "./pkg/mind_wallet.js";

const STAGE_DELAY_MS = 150; // walkthrough card reveal cadence
const REQUIRED_WORDS = 16;

const $ = (sel) => document.querySelector(sel);
const $$ = (sel) => Array.from(document.querySelectorAll(sel));

// ---------- bootstrap ----------

(async function bootstrap() {
  await init();

  populateProfiles();
  hydrateBuildSha();
  wireForm();
  wireKeyboardShortcut();
  wireReveal();
})();

function populateProfiles() {
  const select = $("#profile");
  // Profiles come from the WASM export so adding a profile in Rust auto-
  // populates the UI on the next deploy.
  const profiles = supported_profiles();
  select.innerHTML = "";
  for (const p of profiles) {
    const opt = document.createElement("option");
    opt.value = p;
    opt.textContent = p;
    select.appendChild(opt);
  }
  if (profiles.length > 0) select.value = profiles[0];
}

function hydrateBuildSha() {
  const meta = document.querySelector('meta[name="build-sha"]');
  const sha = (meta && meta.content) || "DEV";
  const slot = document.querySelector('[data-slot="build-sha"]');
  if (slot) slot.textContent = sha.substring(0, 7);
}

// ---------- form wiring ----------

function wireForm() {
  const form = $("#derive-form");
  form.addEventListener("submit", (ev) => {
    ev.preventDefault();
    runDerivation();
  });

  const passphrase = $("#passphrase");
  passphrase.addEventListener("input", () => {
    const err = $("#passphrase-error");
    if (err) err.textContent = "";
  });
}

function wireKeyboardShortcut() {
  document.addEventListener("keydown", (ev) => {
    const isSubmit =
      (ev.metaKey || ev.ctrlKey) && ev.key === "Enter";
    if (!isSubmit) return;
    const target = ev.target;
    if (target && target.tagName === "TEXTAREA") {
      ev.preventDefault();
      runDerivation();
    }
  });
}

function wireReveal() {
  document.addEventListener("click", (ev) => {
    const button = ev.target.closest('[data-action="reveal"]');
    if (!button) return;
    const content = button.closest(".result-content");
    if (content) content.dataset.blur = "false";
  });
}

// ---------- derivation pipeline ----------

function setError(message) {
  const err = $("#passphrase-error");
  if (err) err.textContent = message || "";
}

function runDerivation() {
  const passphrase = $("#passphrase").value.trim();
  const profile = $("#profile").value;

  setError("");

  if (!validate_passphrase(passphrase)) {
    const count = passphrase.length === 0 ? 0 : passphrase.split(/\s+/).length;
    setError(
      `${REQUIRED_WORDS} mots attendus, ${count} reçus. Séparez les mots par des espaces.`
    );
    return;
  }

  const button = $("#derive-button");
  button.disabled = true;
  button.setAttribute("aria-busy", "true");
  resetCards();

  // The synchronous WASM call blocks the main thread for ~600ms on Argon2id
  // m=19MiB. That blocking is deliberate — the walkthrough then animates
  // the real measured timings (see C1' decision in .omc/plans/...).
  try {
    const result = derive(passphrase, profile);
    paintResult(result);
    void animateWalkthrough(result);
  } catch (err) {
    const parsed = parseError(err);
    setError(parsed.message);
  } finally {
    button.disabled = false;
    button.removeAttribute("aria-busy");
  }
}

function parseError(err) {
  if (err && typeof err === "object" && "code" in err) {
    return err;
  }
  return { code: "unknown", message: String(err) };
}

// ---------- walkthrough animation ----------

function resetCards() {
  for (const card of $$(".card")) {
    card.dataset.active = "false";
    for (const slot of card.querySelectorAll("[data-slot]")) {
      slot.textContent = "—";
    }
  }
}

function reveal(card) {
  return new Promise((resolve) => {
    requestAnimationFrame(() => {
      card.dataset.active = "true";
      setTimeout(resolve, STAGE_DELAY_MS);
    });
  });
}

async function animateWalkthrough(result) {
  const reducedMotion =
    window.matchMedia &&
    window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  const stages = [
    {
      sel: ".card--kdf",
      slots: { kdf_preview_hex: result.kdf_preview_hex, kdf_ms: fmtMs(result.kdf_ms) },
    },
    {
      sel: ".card--seed",
      slots: { seed_preview_hex: result.seed_preview_hex, seed_ms: fmtMs(result.seed_ms) },
    },
    {
      sel: ".card--spend",
      slots: { spend_pub_hex: result.spend_pub_hex, keypair_ms: fmtMs(result.keypair_ms) },
    },
    {
      sel: ".card--view",
      slots: { view_pub_hex: result.view_pub_hex, keypair_ms_view: " " },
    },
    {
      sel: ".card--address",
      slots: { address_short: shortAddress(result.address), address_ms: fmtMs(result.address_ms) },
    },
  ];

  for (const stage of stages) {
    const card = document.querySelector(stage.sel);
    if (!card) continue;
    for (const [k, v] of Object.entries(stage.slots)) {
      const slot = card.querySelector(`[data-slot="${k}"]`);
      if (slot) slot.textContent = v;
    }
    if (reducedMotion) {
      card.dataset.active = "true";
    } else {
      // eslint-disable-next-line no-await-in-loop
      await reveal(card);
    }
  }
}

function fmtMs(value) {
  if (typeof value !== "number" || !Number.isFinite(value)) return "—";
  if (value < 1) return "< 1 ms";
  return `${Math.round(value)} ms`;
}

function shortAddress(address) {
  if (!address || address.length < 24) return address || "—";
  return `${address.slice(0, 12)}…${address.slice(-12)}`;
}

// ---------- result panel ----------

function paintResult(result) {
  const wrap = document.querySelector(".result");
  wrap.dataset.state = "ready";
  const content = wrap.querySelector(".result-content");
  content.hidden = false;
  content.dataset.blur = "true";

  setSlot(content, "mnemonic", result.mnemonic);
  setSlot(content, "bundle", result.bundle);
  setSlot(content, "address", result.address);
  setSlot(content, "profile_summary", result.profile_summary);

  try {
    const svg = render_qr_svg(result.bundle);
    const qrSlot = content.querySelector('[data-slot="qr"]');
    if (qrSlot) qrSlot.innerHTML = svg;
  } catch (_) {
    const qrSlot = content.querySelector('[data-slot="qr"]');
    if (qrSlot) qrSlot.textContent = "QR indisponible.";
  }
}

function setSlot(root, name, value) {
  const slot = root.querySelector(`[data-slot="${name}"]`);
  if (slot) slot.textContent = value;
}
