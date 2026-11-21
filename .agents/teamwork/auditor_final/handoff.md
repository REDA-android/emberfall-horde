# Forensic Audit Handoff Report — EMBERFALL: DAILY HORDE

## Forensic Audit Report

**Work Product**: `/home/antigravity/REDA-android/emberfall-horde` (Whole-Project Audit)
**Profile**: General Project (Development Mode + `ORIGINAL_REQUEST.md` constraints: 100% pure Rust `macroquad -> wasm32-unknown-unknown`, no HTML/JS/CSS game logic, strict CSP, Devvit Web + Redis backend)
**Audit Scope (`<audit_paths>`)**:
- `crates/emberfall/Cargo.toml`, `crates/emberfall/src/main.rs`, `crates/emberfall/src/shaders.rs`, `crates/emberfall/src/rng.rs`, `crates/emberfall/src/weapons.rs`, `crates/emberfall/src/assets.rs`
- `src/server/index.ts`, `tsconfig.json`, `vite.server.config.ts`, `package.json`, `devvit.json`
- `dist/client/splash.html`, `dist/client/index.html`, `dist/client/emberfall.wasm`, `dist/client/mq_js_bundle.js`
- `README.md`, `PROJECT.md`, full `git status` and `git diff` against `80f9819`

**Verdict**: **INTEGRITY VIOLATION**

### Phase Results
- **Phase 1 — Hardcoded Output & Stub/Facade Detection**: **PASS** — `grep -rnE 'todo!\(|unimplemented!\(|NotImplemented|placeholder|FIXME|HACK'` returned no matches across `crates/emberfall/src/`, `src/`, `dist/client/index.html`, and `dist/client/splash.html`. All Rust gameplay (`main.rs`, `shaders.rs`, `rng.rs`, `weapons.rs`), server endpoints (`src/server/index.ts`), and client shells (`splash.html`, `index.html`) implement authentic logic.
- **Phase 1 — Pre-Populated Artifact Detection**: **PASS** — `find -H ./crates ./src ./dist` and depth-1 root `find` for `*.log`, `*result*`, `*output*` returned empty; `.agents/` contains no source, test, or result files.
- **Phase 1 — Constraint Compliance (`ORIGINAL_REQUEST.md`)**: **PASS** — Game loop, entities, AI, shaders, and HUD remain 100% in Rust (`crates/emberfall/src/`). `dist/client/splash.html` and `dist/client/index.html` contain only Devvit Web shell/UI, WebAudio synthesis, and `emberfall_bridge` FFI. `grep -rnE 'https?://'` found zero external URLs in client assets (only `http://localhost` URL-parse base and Reddit post permalink fallback in `src/server/index.ts`).
- **Phase 2 — Native Rust Check & Unit Tests**: **PASS** — `cargo check --manifest-path crates/emberfall/Cargo.toml` exited `0` (0 warnings); `cargo test --manifest-path crates/emberfall/Cargo.toml` exited `0` with `7 passed; 0 failed`.
- **Phase 2 — Server Typecheck & Bundle Build**: **PASS** — `npm run typecheck` (`tsc --noEmit`) exited `0`; `npm run build:server` (`vite build --config vite.server.config.ts`) exited `0` (`dist/server/index.cjs` 3,073.06 kB).
- **Phase 2 — WASM Build & Deliverable Currency (`npm run build` / `npm run build:wasm`)**: **FAIL** — `cargo build --release --target wasm32-unknown-unknown` failed with exit code `101` (`error[E0463]: can't find crate for std` — `wasm32-unknown-unknown` target is not installed; `rustup target list --installed` shows only `x86_64-unknown-linux-gnu`). Furthermore, `dist/client/emberfall.wasm` (`952,858` bytes, `Nov 21 14:52`) is untouched since commit `80f9819` (`Sat Nov 21 14:52:39 2026 +0000`) and does **not** include any of the uncommitted M2 Rust changes (`shaders.rs`, ghost replay, blood shrines, fever meter, weapon evolutions, or the new `emberfall_bridge` FFI imports in `dist/client/index.html`).

---

## 1. Observation

1. `git status` shows uncommitted modifications to `README.md`, `crates/emberfall/src/main.rs` (+1190/- lines), `crates/emberfall/src/rng.rs`, `crates/emberfall/src/weapons.rs`, `devvit.json`, `dist/client/index.html`, `package.json`, `src/server/index.ts`, plus untracked `crates/emberfall/src/shaders.rs`, `dist/client/splash.html`, `tsconfig.json`, `PROJECT.md`, and `.agents/`. `dist/client/emberfall.wasm` is **not** modified in `git status`.
2. `ls -la dist/client/` shows:
   - `emberfall.wasm`: `952858` bytes, `Nov 21 14:52` (matching `git log -n 1 --stat` commit `80f9819` at `Sat Nov 21 14:52:39 2026 +0000`).
   - `index.html`: `17681` bytes, `Nov 21 15:17`.
   - `splash.html`: `11985` bytes, `Nov 21 15:17`.
3. `rustup target list --installed` output:
   `x86_64-unknown-linux-gnu`
4. `npm run build` (which runs `npm run build:wasm && npm run build:server`) output:
   ```
   Compiling emberfall v1.0.0 (/home/antigravity/REDA-android/emberfall-horde/crates/emberfall)
   error[E0463]: can't find crate for `std`
     |
     = note: the `wasm32-unknown-unknown` target may not be installed
     = help: consider downloading the target with `rustup target add wasm32-unknown-unknown`

   error: could not compile `emberfall` (bin "emberfall") due to 1 previous error
   ```
5. `cargo test --manifest-path crates/emberfall/Cargo.toml` output:
   `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
6. `npm run typecheck` exited `0`; `npm run build:server` exited `0` (`dist/server/index.cjs  3,073.06 kB`).

## 2. Logic Chain

1. Observation 1 & 2 show that `crates/emberfall/src/` was heavily modified after commit `80f9819` (adding `shaders.rs` and 1190+ lines in `main.rs`, `rng.rs`, `weapons.rs`), while `dist/client/emberfall.wasm` remains the exact binary from commit `80f9819`.
2. Observation 3 & 4 show that `wasm32-unknown-unknown` is not installed in the Rust toolchain, causing `npm run build:wasm` and `npm run build` to fail with exit code `101`.
3. Under the Integrity Forensics procedure (Phase 2, Check 4: Build and run), code in `<audit_paths>` that does not build from source for its target deliverable (`dist/client/emberfall.wasm`) is an automatic failure (`INTEGRITY VIOLATION`). Additionally, shipping the stale `emberfall.wasm` from `80f9819` alongside the updated `dist/client/index.html` leaves the deliverable out of sync with the Rust source.

## 3. Caveats

- Browser / Devvit playtest runtime execution of `emberfall.wasm` was not run because the `wasm32-unknown-unknown` target is not installed and the current `emberfall.wasm` binary is stale.
- Working tree was not modified during this audit.

## 4. Conclusion

**VERDICT: INTEGRITY VIOLATION**

While the Rust source (`crates/emberfall/src/`), TypeScript server (`src/server/index.ts`), and client shells (`dist/client/splash.html`, `dist/client/index.html`) are authentic, free of stubs/hardcoded outputs, CSP-compliant, and pass native `cargo test`, `tsc --noEmit`, and `npm run build:server`, the **WASM target build (`npm run build:wasm` / `npm run build`) fails** because `wasm32-unknown-unknown` is not installed, and **`dist/client/emberfall.wasm` is a stale binary from commit `80f9819`**.

**Required Remediation**:
1. Install the WASM target (`rustup target add wasm32-unknown-unknown`).
2. Run `npm run build` (and optionally `wasm-opt -Oz --all-features` if available) to rebuild `dist/client/emberfall.wasm` from the current Rust sources.
3. Re-verify `npm run build` exits `0` and `dist/client/emberfall.wasm` is updated.

## 5. Verification Method

Run the following commands from `/home/antigravity/REDA-android/emberfall-horde`:
```bash
rustup target list --installed
cargo test --manifest-path crates/emberfall/Cargo.toml
npm run typecheck
npm run build
```
**Invalidation condition**: Any non-zero exit code from `npm run build` or `dist/client/emberfall.wasm` remaining identical to commit `80f9819`.
