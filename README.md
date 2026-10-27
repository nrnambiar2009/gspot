# GSpot

![GSpot beside a TextEdit draft: the Review panel lists the suggestions, the draft is underlined in place, and a card offers the fix](docs/hero.png)

A local-first, open-source Grammarly-style writing assistant for macOS, with a Linux port in progress
(Ubuntu 26.04, GNOME on Wayland; see [Linux status](#linux-status)). On macOS a floating panel docked to the
left edge of the screen follows the text field you are typing in (any app), underlines issues in place,
and runs the full sidebar of tools: suggestions, AI chat, plagiarism, tone, performance score,
paraphraser, humanizer, reader reactions, translate, explain it, AI rewriter, AI detector.

The panel starts **tucked**: a small G tab sits flush with the left screen edge. Click the tab and the
panel slides out; the red close button or the "Tuck away" chevron at the bottom of the rail slides it
back. The tab shows the current suggestion count. (Grammarly tucks into the right edge; GSpot
tucks into the left. `dock_side` in Settings flips it.)

* Correctness engine: [Harper](https://writewithharper.com) (`harper-core`, in-process, offline).
  A sentence over 40 words gets a "Split the long sentence" card with a lossless split ready to accept;
  the spot check (the model) replaces it with a better split when one passes the guards.
* Text access: the macOS Accessibility API (focused field, selection, `AXBoundsForRange` geometry).
  Terminals (Terminal, iTerm2, Warp, Ghostty, kitty, Alacritty, WezTerm, Hyper) are skipped: a shell
  transcript is not writing. Any other app can be skipped from the Review view ("Skip this app") and
  turned back on under Settings → Skipped apps.
* Replacements: AX selected-text set, then a verified pasteboard patch fallback.
* AI features: **publik API** by default (no account, no key: the app mints its own key on this
  computer after a two-sentence disclosure; you sign in with publik to add free use or a plan),
  or your own key (paste it; OpenAI, Anthropic, OpenRouter, Groq and Google are detected from the
  prefix and a model is picked for you). URLs, ports and model names sit under "Advanced". There
  is no local-model option; only the e2e rig points the app at Ollama, through the debug probe.
* Spot check: sentence-level suggestions (make it concise, active voice, rephrase for clarity,
  remove a repeated or filler sentence) from the model, guarded so numbers, names and quotes
  survive and clean prose is left alone. See `docs/SPOTCHECK.md`.
* Plagiarism: web search through publik API when it is on (`POST /search`, one cent a search), your
  own Brave Search API key if you paste one in Settings, else a keyless chain (Brave HTML, Bing,
  DuckDuckGo); 7-gram overlap, APA/MLA/Chicago citations.

See `docs/REVERSE-ENGINEERING.md` for what was learned from Grammarly Desktop and `docs/PLAN.md` for the gates.

## Linux status

The Linux port follows the gates in the port plan. **Only gate L0 is done**, so this is a developer
preview, not a drop-in replacement for the macOS app.

| Gate | What | State |
|---|---|---|
| L0 | Compiles on Linux; panel opens; Harper runs on text typed into the panel | done |
| L1 | Read the focused field of other apps through AT-SPI | not started |
| L2 | Accept a suggestion in the other app (AT-SPI edit, paste fallback), verified by read-back | not started |
| L3 | Underlines in place where the compositor gives valid coordinates, else panel-only | not started |
| L4 | Autostart, tray handling, `.deb` package | not started |

What works today: the panel, the Harper correctness engine and the unit tests. The AI features (spot check, chat, rewriter and the
rest) are not available on Linux yet. The Review view has a text box on Linux: type or paste text and the suggestions appear, and
Accept edits that text. What does not work yet: following the text field of another app, replacing text in
other apps, underlines in place, the tray icon on stock GNOME, and a `.deb`.

The window layer runs under XWayland (`GDK_BACKEND=x11`) because GNOME on Wayland lets an app neither place
nor stack its own windows globally. Expect in-place underlines to work in X11 apps and some Wayland apps;
the panel with click-to-accept suggestions is the fallback everywhere else.

### Build and run on Linux

Needs Rust (via [rustup](https://rustup.rs)), Node 22, and pnpm.

```bash
sudo apt install build-essential curl wget file libwebkit2gtk-4.1-dev libxdo-dev libssl-dev \
  libayatana-appindicator3-dev librsvg2-dev libatspi2.0-dev
pnpm install
cd src-tauri && ../ui/node_modules/.bin/tauri build --debug --no-bundle
GDK_BACKEND=x11 ./target/debug/grammarlama
```

`pnpm dev` gives hot reload. Set `GRAMMARLAMA_DEBUG_PORT=47811` to get the loopback probe; on Linux
`curl -XPOST localhost:47811/check -d '{"text":"..."}'` lints text as if it were typed in the panel, and
`/state` and `/accept` work on it.

## License

AGPL-3.0, see `LICENSE`. If you distribute a modified build or let others use it over a network, you must
publish your source under the same license. The UI imitates Grammarly's look; rebrand it before you
distribute your own build.

## Install (macOS)

GSpot is listed on [publik](https://publikhq.com/gspot).

1. Download `GSpot_<version>_aarch64.dmg` from the [latest GitHub release](https://github.com/Blueturboguy07/gspot/releases/latest) (Apple silicon Macs).
2. Open the dmg and drag GSpot to Applications.
3. Open GSpot. The app is signed with a Developer ID and notarized by Apple, so macOS opens it without a warning.
4. Grant Accessibility when asked, under System Settings, Privacy & Security, Accessibility.

GSpot lives in the menu bar and as a tab on the left screen edge. AI features run through publik API by
default; the first use shows a two-sentence disclosure and mints a key for this computer.

## Build (macOS)

```bash
pnpm install        # installs ui/ as well (postinstall)
pnpm dev            # hot reload: Vite + the app
# signed release (.app + .dmg)
scripts/build-debug.sh                   # debug build with the publik app token compiled in (for the e2e gates)
scripts/build-macos.sh --skip-notarize   # or with NOTARY_PROFILE / APPLE_ID+APPLE_PASSWORD+APPLE_TEAM_ID
scripts/verify-macos.sh                  # codesign, Gatekeeper, launch
```

Grant Accessibility to GSpot once (System Settings → Privacy & Security → Accessibility).
The publik app token is read from `PUBLIK_APP_TOKEN` or `~/.publik/gspot-app-token` at build
time (minted with publik's `scripts/mint-app-token.mts gspot`); a build without it still
offers your own key. `PUBLIK_SELF_TEST=1 /Applications/GSpot.app/Contents/MacOS/grammarlama` prints one
line that proves the publik API wiring of a built app.

## Test

The e2e scripts below drive TextEdit and need macOS. On Linux, run the two unit test suites.

```bash
cd ui && pnpm test                         # TypeScript unit tests
cd src-tauri && cargo test                 # Rust unit tests (engine, score, phrases, plagiarism, …)
tests/e2e/host_textedit.sh                 # reads/lints/fixes a real TextEdit document through AX (incl. a spot-check accept)
tests/e2e/features.sh                      # every LLM/web feature against the rig's Ollama + the live web
tests/e2e/spotcheck.sh                     # the 14-document spot-check corpus through the app (>= 90 % to pass)
tests/tuning/live_corpus.sh                # the same corpus through the pipeline alone (no app windows)
tests/e2e/setup.sh                         # publik API provisioning, own-key probe, the rig's Ollama
```

The e2e scripts launch the debug binary with `GRAMMARLAMA_DEBUG_PORT=47331` and talk to a loopback
probe (`/state`, `/accept`, `/feature`, `/lint`). `start_app` then points the app at the rig's Ollama
(`qwen2.5:7b`, or `E2E_MODEL`) through the probe, because the product itself offers no local model;
`E2E_KEEP_LLM=1` runs the gates on whatever the app is set to. A debug (`cargo build`) binary loads the Vite dev
server, which the harness starts for you. To test the signed bundle instead:

```bash
APP_BIN=src-tauri/target/release/bundle/macos/GSpot.app/Contents/MacOS/grammarlama tests/e2e/host_textedit.sh
```

Accessibility trust: a binary launched from a terminal inherits the terminal's grant (macOS charges
the "responsible process"). Launched from Finder, GSpot asks for its own grant once.
