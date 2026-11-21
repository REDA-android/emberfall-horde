# Reviewer 1 (Primary Verifier) Handoff Report — M1 + M2 Verification Wave

## Review Summary

**Verdict**: **APPROVE**

**Overall Risk Assessment (Adversarial)**: **LOW** (for M1 + M2 scope; `wasm32-unknown-unknown` build & live `devvit playtest` remain for M3).

No integrity violations were detected. All M1 and M2 features are genuinely implemented, all runnable full-suite checks pass, strict CSP compliance holds (`0` external `http://`/`https://` URLs in client HTML), and 100% of HTTP and FFI interface contracts in `PROJECT.md` align across Rust, JS, and TypeScript.

---

## 1. Observation

### A. Full-Suite Build, Test & CSP Verification (Independently Executed)
1. `npm run typecheck` (`tsc --noEmit`):
   - **Exit 0**, no diagnostics.
2. `npm run build:server` (`vite build --config vite.server.config.ts`):
   - **Exit 0**, `758 modules transformed`, produced `dist/server/index.cjs` (`3,073.06 kB`) in `4.08s`.
3. `cargo check --manifest-path crates/emberfall/Cargo.toml`:
   - **Exit 0**, 1 warning: `method 'accent' is never used` at `crates/emberfall/src/weapons.rs:92`.
4. `cargo test --manifest-path crates/emberfall/Cargo.toml`:
   - **Exit 0**, `7 passed; 0 failed; 0 ignored`:
     - `rng::tests::deterministic_365_days_and_ranks`
     - `rng::tests::fever_multiplier_bounds`
     - `rng::tests::deterministic_shrines_per_seed`
     - `rng::tests::mutator_index_roundtrip`
     - `weapons::tests::build_summary_contains_levels_and_evolutions`
     - `weapons::tests::roll_three_is_deterministic_and_distinct`
     - `weapons::tests::weapon_evolutions_require_lv3_pair`
5. CSP URL check (`grep -nE 'https?://' dist/client/splash.html dist/client/index.html`):
   - **Exit 1 (no matches)** -> `CSP_CHECK_PASSED`. Zero external `http://` or `https://` URLs in either HTML file.

### B. HTTP Interface Contract Alignment (`PROJECT.md § 1`)
Verified `src/server/index.ts` against `dist/client/splash.html` and `dist/client/index.html`:
- `GET /api/init` (`src/server/index.ts:289-358` <-> `splash.html:380`, `index.html:170`): Returns all 12 contract fields (`seed`, `mutatorOverride`, `username`, `streak`, `personalBest`, `myRank`, `totalPlayers`, `bossKillsToday`, `siegeTarget`, `leaderboard`, `topGhost`, `canVoteMutator`).
- `POST /api/start-run` (`src/server/index.ts:360-379` <-> `index.html:276`): Returns `{ ok: true, runId, startedAt }`, cached in `currentRunId`.
- `POST /api/submit-run` (`src/server/index.ts:381-520` <-> `index.html:290-327`): Accepts `{ runId, score, kills, bossSlain, level, buildSummary, ghostPoints }`, validates `runId` elapsed `10_000ms..180_000ms`, rate-limits 10s NX, clamps bounds, updates `ef:lb`, `ef:meta`, `ef:streak`, `ef:bosses`, `ef:ghost`, and returns `{ ok, rank, myRank, totalPlayers, streak, bossKillsToday, isNewBest, canVoteMutator }`.
- `POST /api/share-comment` (`src/server/index.ts:557-635` <-> `index.html:337-370`): Accepts all 9 contract fields, posts Wordle-style Markdown + emoji build card as `USER`, returns `200 { ok: true }` or `400/401/429 { ok: false, error }`.
- `POST /api/vote-mutator` (`src/server/index.ts:522-555` <-> `index.html:374-393`): Accepts `{ mutatorIndex: 0..3 }`, enforces top-3 daily rank (`403` otherwise), stores in `ef:mutator-vote:{sub}:{tomorrow}`, returns `{ ok: true, mutatorIndex }`.

### C. FFI Bridge Alignment (`PROJECT.md § 2`)
Verified all 16 `extern "C"` functions in `crates/emberfall/src/main.rs:14-52` (`wasm32`) and `main.rs:54-126` (native stubs) against `dist/client/index.html:240-457` (`miniquad_add_plugin` `'emberfall_bridge'`):
- `emberfall_get_day_number() -> u32` (`main.rs:15,55` <-> `index.html:244`)
- `emberfall_play_sfx_pitched(kind: i32, pitch_mul: f32)` (`main.rs:16,59` <-> `index.html:253`; `emberfall_play_sfx` alias kept at `index.html:249`)
- `emberfall_set_muted(muted: i32)` (`main.rs:17,61` <-> `index.html:257`)
- `emberfall_toggle_drone(active: i32)` (`main.rs:18,63` <-> `index.html:266`)
- `emberfall_on_run_start()` (`main.rs:19,65` <-> `index.html:272`)
- `emberfall_on_run_complete(score: u32, kills: u32, boss_slain: i32, level: u32, build_ptr: *const u8, build_len: u32, ghost_ptr: *const i16, ghost_count: u32)` (`main.rs:20-29,67-77` <-> `index.html:284`)
- `emberfall_share_comment(score: u32, kills: u32, boss_slain: i32, level: u32, build_ptr: *const u8, build_len: u32)` (`main.rs:30-37,79-87` <-> `index.html:330`)
- `emberfall_vote_mutator(mutator_idx: i32)` (`main.rs:38,89` <-> `index.html:372`)
- `emberfall_poll_init_ready() -> i32` (`main.rs:39,91` <-> `index.html:396`)
- `emberfall_get_init_field(field_id: i32) -> i32` (`main.rs:40,95` <-> `index.html:400`, cases `0..=9` match `main.rs:467-496`)
- `emberfall_copy_str_field(field_id: i32, index: i32, out_ptr: *mut u8, max_len: u32) -> u32` (`main.rs:41-46,99-106` <-> `index.html:416`)
- `emberfall_get_lb_score(index: i32) -> u32` (`main.rs:47,108` <-> `index.html:430`)
- `emberfall_get_ghost_x(index: i32) -> i32` & `emberfall_get_ghost_y(index: i32) -> i32` (`main.rs:48-49,112-118` <-> `index.html:435-445`)
- `emberfall_get_share_state() -> i32` & `emberfall_get_submit_state() -> i32` (`main.rs:50-51,120-126` <-> `index.html:447-453`)

### D. M1 & M2 Feature Inventory & Minor Findings
All M1 (#1–#8, plus #9 TS toolchain) and M2 (#10–#15) features are genuinely implemented (`devvit.json`, `src/server/index.ts`, `splash.html`, `index.html`, `shaders.rs`, `rng.rs`, `weapons.rs`, `main.rs`).

Minor non-blocking findings:
1. **[Minor] Dead code warning**: `crates/emberfall/src/weapons.rs:92` — `EvolutionId::accent` is never called, causing 1 compiler warning.
2. **[Minor] Share error label & retry**: `crates/emberfall/src/main.rs:2511,2520` — `share_state == 3` always displays `"LOGIN ON REDDIT TO SHARE"` (even on 429/network errors) and `share_state == 0` prevents retrying until a new run starts.
3. **[Minor] Fixed 96-byte FFI string buffer**: `crates/emberfall/src/main.rs:141-145` — `read_ffi_str` uses `[0u8; 96]`; `String::from_utf8_lossy` handles mid-codepoint truncation safely without panicking, though messages >96 bytes truncate.

---

## 2. Logic Chain
1. Independent execution of `npm run typecheck`, `npm run build:server`, `cargo check`, `cargo test`, and the CSP regex check all exited `0` with no errors or external URLs.
2. Line-by-line comparison of `src/server/index.ts`, `splash.html`, `index.html`, and `main.rs` confirms 100% alignment of all 5 HTTP endpoints and all 16 FFI signatures (including synchronous `wasm_memory.buffer` reads in `readUtf8`/`readGhostPoints` before `fetch()`).
3. Inspection of `shaders.rs`, `rng.rs`, `weapons.rs`, and `main.rs` confirms genuine implementation of GLSL100 shaders (`#version 100`, `precision mediump float;`), `render_target` post-processing, hitstop, quadratic trauma shake, Blood Shrines, Weapon Evolutions, Fever Meter, Ghost Replay, Pause/Mute controls, and Social HUD.
4. No critical/major defects or integrity violations exist; therefore, the verdict is **APPROVE**.

---

## 3. Caveats
- `wasm32-unknown-unknown` target is not installed in this VM's read-only `rustup` environment, so `cargo check --target wasm32-unknown-unknown` and rebuilding `dist/client/emberfall.wasm` could not be executed here (scoped to M3).
- Live `devvit playtest` execution against Reddit's WebView host was not run in this sandbox.

---

## 4. Conclusion
**APPROVE** — M1 and M2 pass verification. Proceed to M3 (`wasm32-unknown-unknown` build, `wasm-opt`, and final whole-project integration).

---

## 5. Verification Method
Run in `/home/antigravity/REDA-android/emberfall-horde`:
```bash
npm run typecheck
npm run build:server
cargo check --manifest-path crates/emberfall/Cargo.toml
cargo test --manifest-path crates/emberfall/Cargo.toml
! grep -nE 'https?://' dist/client/splash.html dist/client/index.html && echo "CSP_CHECK_PASSED"
```
