# Forensic Integrity Audit Report (M1 + M2 Intermediate Audit)

**Work Product**: EMBERFALL: DAILY HORDE — M1 & M2 Deliverables  
**Audit Scope (`<audit_paths>`)**:
- `src/server/index.ts`, `devvit.json`, `tsconfig.json`, `package.json`, `dist/client/splash.html`, `dist/client/index.html`
- `crates/emberfall/src/shaders.rs`, `crates/emberfall/src/rng.rs`, `crates/emberfall/src/weapons.rs`, `crates/emberfall/src/main.rs`

**Profile**: General Project (Inferred Mode: **Demo Mode** from `ORIGINAL_REQUEST.md` — 100% pure Rust client via `macroquad` -> `wasm32-unknown-unknown`, Devvit Web + Redis backend, strict CSP, no HTML/JS/CSS game logic)  
**Verdict**: **CLEAN**

---

## 1. Observation

### Git & Workspace Hygiene
- `git status --short`: Modified `crates/emberfall/src/main.rs`, `crates/emberfall/src/rng.rs`, `crates/emberfall/src/weapons.rs`, `devvit.json`, `dist/client/index.html`, `package.json`, `src/server/index.ts`; Untracked `PROJECT.md`, `crates/emberfall/src/shaders.rs`, `dist/client/splash.html`, `tsconfig.json`, `.agents/`.
- `find .agents -type f | sort`: Contains only agent metadata (`ORIGINAL_REQUEST.md`, `BRIEFING.md`, `DISPATCH.md`, `progress.md`, `handoff.md`). Zero source, test, fixture, or data files inside `.agents/`.
- Pre-populated artifact scan (`find -H './src/server' './dist/client' './crates/emberfall/src' ...` and depth-1 `'.'`): Exit code `0`, no output (no `.log`, `*result*`, or `*output*` artifacts).
- External URL / CSP check (`grep -rn 'https\?://' dist/client/splash.html dist/client/index.html src/server/index.ts`): Zero external URLs in `dist/client/splash.html` and `dist/client/index.html`.

### Builds & Tests Executed
1. `npm run typecheck && npm run build:server` → **PASS** (exit code `0`):
   - `tsc --noEmit` succeeded with 0 errors.
   - `vite build --config vite.server.config.ts` transformed 758 modules and built `dist/server/index.cjs` (`3,073.06 kB`).
2. `cargo test --manifest-path crates/emberfall/Cargo.toml` → **PASS** (exit code `0`):
   - `7 passed; 0 failed; 0 ignored` (`deterministic_shrines_per_seed`, `deterministic_365_days_and_ranks`, `fever_multiplier_bounds`, `mutator_index_roundtrip`, `build_summary_contains_levels_and_evolutions`, `roll_three_is_deterministic_and_distinct`, `weapon_evolutions_require_lv3_pair`).
   - 1 dead-code warning (`EvolutionId::accent` in `src/weapons.rs:92`).
3. `cargo check --manifest-path crates/emberfall/Cargo.toml --target wasm32-unknown-unknown` → **ENV MISSING** (exit code `101`):
   - `error[E0463]: can't find crate for std` / `core` (`rustup target list --installed` shows only `x86_64-unknown-linux-gnu`; `wasm32-unknown-unknown` target is not yet installed, which is scheduled in `PROJECT.md` under M3).

### Phase Results (Source & Authentic Logic Verification)
- **Hardcoded test results**: **PASS** — No hardcoded test outputs or PASS/FAIL strings in source or tests.
- **Facade implementations**: **PASS** — All M1 & M2 features implement genuine logic:
  - **Scheduler Lock (`src/server/index.ts:150-191`)**: `createDailyPost({ respectLock: true })` checks `ef:post:{sub}:{seed}`, acquires atomic `redis.set(lockKey, '1', { nx: true, expiration: 600_000 })`, rechecks `postKey`, creates post via `reddit.submitCustomPost`, and sets 30-day TTL (`THIRTY_DAYS_SEC = 30 * 86400`). `onPostDelete` (`src/server/index.ts:250-287`) cleans up post-scoped keys and share guards.
  - **Run Session Anti-Cheat (`src/server/index.ts:360-488`)**: `/api/start-run` issues a 16-byte random hex `runId` with 300s Redis TTL; `/api/submit-run` validates `runId` elapsed time (`10_000..180_000` ms) and deletes it on use, enforces 10s per-user `NX` rate limit, and clamps `score` (`0..500000`), `kills` (`0..10000`), `level` (`1..99`), and `ghostPoints` (`<=200` pairs clamped to `i16`).
  - **Devvit Splash & Audio/FFI Bridge (`dist/client/splash.html`, `dist/client/index.html`, `devvit.json`)**: `splash.html` (`11,985` bytes) has zero WASM and zero game logic, fetches `/api/init`, and invokes `requestExpandedMode(event, 'game')`. `index.html` registers `miniquad_add_plugin` (`emberfall_bridge`) matching all `extern "C"` imports in `crates/emberfall/src/main.rs:14-52`, unlocks WebAudio synchronously on `pointerdown`/`touchstart`/`keydown`, and delegates all gameplay to `emberfall.wasm`.
  - **Weapon Evolutions (`crates/emberfall/src/weapons.rs:68-191`, `main.rs:1151-1286`, `1564-1569`)**: All 3 evolutions (`MeteorBarrage`, `CrimsonReaper`, `InfernoVortex`) require Lv.3+ on both paired upgrades and actively modify projectile count/pierce/speed, slash radius/damage/lifesteal, and orbit count/spin/radius in the combat loop.
  - **Blood Shrines (`crates/emberfall/src/rng.rs:103-115`, `main.rs:1063-1105`)**: Deterministic per-seed placement (1–2 shrines), 2.0s channeling mechanic, grants `+0.5x` `shrine_score_mult`, spawns an elite `Golem`/`Wraith`, and triggers a bonus `LevelUp` choice.
  - **#1 Champion Ghost Replay (`crates/emberfall/src/main.rs:1006-1012`, `1764-1809`)**: Records `(x, y)` `i16` samples at 2 Hz (up to 200 points), submits via `emberfall_on_run_complete`, and interpolates/renders the translucent golden phantom with `u/<ghost_username>`.
  - **GLSL100 Shaders & Juice (`crates/emberfall/src/shaders.rs:1-138`, `main.rs:1880-1899`, `2632-2670`)**: `#version 100` + `precision mediump float;` shaders for `hit_flash` (`u_flash`) on damaged enemies and `render_target` `post_process` (`u_time`, `u_low_hp`, `u_aberration`, `u_shockwave`, `u_blood_moon`), plus `hitstop`, quadratic `trauma * trauma` camera shake, and sprite squash/stretch.
- **Pre-populated verification artifacts**: **PASS** — None found.
- **Dependency / CSP audit (Demo Mode)**: **PASS** — Only `@devvit/web`, `macroquad`, and `node:crypto`/`node:http` used; zero external URLs in client HTML.

---

## 2. Logic Chain

1. `ORIGINAL_REQUEST.md` specifies a 100% pure Rust `macroquad` client compiled to `wasm32-unknown-unknown`, Devvit Web + Redis backend, strict CSP, and zero HTML/JS/CSS game logic → **Demo Mode** applies.
2. Static inspection of `dist/client/splash.html`, `dist/client/index.html`, `src/server/index.ts`, `crates/emberfall/src/{shaders,rng,weapons,main}.rs` confirms all game logic resides in Rust, client HTML/JS acts strictly as Devvit splash/FFI/WebAudio shell with no external URLs, and server endpoints implement real Redis persistence, locks, TTLs, and anti-cheat validation.
3. Empirical execution confirms `tsc --noEmit`, `vite build --config vite.server.config.ts`, and `cargo test` (7/7) all pass cleanly, and no pre-populated result artifacts or `.agents/` layout violations exist.
4. Therefore, all M1 + M2 deliverables in `<audit_paths>` are authentic and **CLEAN**.

---

## 3. Caveats

1. **`wasm32-unknown-unknown` target not installed in VM**: `cargo check --target wasm32-unknown-unknown` failed with `E0463` because `rustup` only has `x86_64-unknown-linux-gnu` installed. Per `PROJECT.md`, installing `wasm32-unknown-unknown` and rebuilding `dist/client/emberfall.wasm` is part of **M3**.
2. **Optional `runId` in `/api/submit-run`**: `src/server/index.ts:406` validates `runId` when present, which matches the `PROJECT.md` interface contract (`Validates runId (if present, elapsed 10s..180s)`), though omitting `runId` is still accepted if rate-limited.
3. **Runtime WebGL/Devvit Execution**: Live browser WebGL shader compilation and Devvit playtest runtime were not executed in this intermediate audit (deferred to M3 / playtest).

---

## 4. Conclusion

**VERDICT: CLEAN** — M1 and M2 deliverables pass all Forensic Integrity checks under Demo Mode with no hardcoded outputs, no facade implementations, no fabricated artifacts, and full alignment with `ORIGINAL_REQUEST.md` and `PROJECT.md`. Ready to proceed to M3 (`wasm32-unknown-unknown` toolchain setup, `emberfall.wasm` rebuild, and final whole-project gate).

---

## 5. Verification Method

Run from `/home/antigravity/REDA-android/emberfall-horde`:
```bash
npm run typecheck
npm run build:server
cargo test --manifest-path crates/emberfall/Cargo.toml
grep -rn 'https\?://' dist/client/splash.html dist/client/index.html || true
```
- **Invalidation conditions**: Any failure in `tsc --noEmit`, `vite build`, or `cargo test`, or any external URL in `dist/client/*.html`, invalidates this verdict.
