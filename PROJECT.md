# Project: EMBERFALL: DAILY HORDE

## Architecture
- **100% Pure Rust Client (`crates/emberfall/`)**: `macroquad 0.4` compiled to `wasm32-unknown-unknown` (`dist/client/emberfall.wasm`), zero HTML/JS/CSS game logic, all PNGs embedded via `include_bytes!()`.
- **Devvit Web Shell (`dist/client/` + `src/client/`)**:
  - `splash.html` (Devvit `default` inline feed entrypoint): lightweight `<30 KB` zero-WASM dark fantasy card fetching `/api/init` and calling `requestExpandedMode(event, 'game')` on click.
  - `index.html` (Devvit `game` expanded fullscreen entrypoint): loads `mq_js_bundle.js` + `emberfall.wasm`, registers `emberfall_bridge` plugin for WebAudio SFX/drone, UTC day/init state, run lifecycle (`/api/start-run`, `/api/submit-run`), ghost upload/download, mutator voting, and Reddit comment sharing (`/api/share-comment`).
- **Devvit Web + Redis Backend (`src/server/index.ts` -> `dist/server/index.cjs`)**:
  - Automated UTC daily post scheduler (`scheduler.tasks` + atomic `SET NX` lock).
  - Privacy-compliant Redis schema (`30 * 86400`s TTLs + `onPostDelete` cleanup).
  - Run session anti-cheat (`/api/start-run` -> `/api/submit-run` duration & plausibility checks + rate limit).
  - Social & community hooks: Top-10 + self-rank leaderboard, daily streaks, Subreddit Collective Siege Goal, #1 Champion Ghost Replay (`~1.4 KB` 2Hz path), Winner-Picks-Tomorrow's-Mutator, and Wordle-style Emoji Build Share Cards.

## Code Layout
- `crates/emberfall/Cargo.toml` — Rust crate manifest (`emberfall v1.0.0`, `macroquad = "0.4"`, size-optimized release profile)
- `crates/emberfall/src/main.rs` — Core game loop, FFI bridge, entities, HUD, ghost replay, shrines, fever meter, juice (`hitstop`, `trauma`, squash/stretch)
- `crates/emberfall/src/shaders.rs` — GLSL100 (`#version 100`, `precision mediump float;`) hit-flash sprite material & `render_target` post-processing (vignette, low-HP pulse, chromatic aberration, shockwave)
- `crates/emberfall/src/weapons.rs` — Upgrades, Weapon Evolutions (synergy forms), HeroBuild
- `crates/emberfall/src/rng.rs` — Deterministic FNV-1a + `xorshift64*` PRNG, `DailyModifier`, `compute_rank`
- `crates/emberfall/src/assets.rs` — Compile-time `include_bytes!()` textures
- `crates/emberfall/assets/` — Custom dark fantasy sprites + Kenney CC0 VFX PNGs
- `src/server/index.ts` — `@devvit/web` Express/Node server + Redis API + triggers + scheduler
- `dist/client/splash.html` — Lightweight inline Devvit feed splash card
- `dist/client/index.html` — Expanded fullscreen WASM host + WebAudio + `emberfall_bridge`
- `devvit.json` — Devvit Web manifest (`default` splash + `game` expanded, `scheduler.tasks`, `onPostDelete`, `onAppInstall`)
- `tsconfig.json` — TypeScript config for `src/server/index.ts`
- `package.json` — Build scripts (`build:wasm`, `build:server`, `build`, `typecheck`)

## Feature Inventory
Every Survey feature assigned a milestone (none unassigned):
| # | Feature | Description | Milestone | Source |
|---|---------|-------------|-----------|--------|
| 1 | Inline Splash + `requestExpandedMode` | Split `devvit.json` into `default` (`splash.html`, zero-WASM) and `game` (`index.html`, expanded fullscreen) | M1 | Explorer 2 (P0) |
| 2 | UTC Daily Scheduler + Lock | `scheduler.tasks` hourly/daily cron -> `/internal/scheduler/daily-post` with Redis `SET NX` lock | M1 | Explorer 2 (P0) |
| 3 | Privacy TTLs + `onPostDelete` | 30-day `redis.expire` on daily keys + `onPostDelete` trigger to purge post-scoped data | M1 | Explorer 2 (P0) |
| 4 | Run Session Anti-Cheat | `POST /api/start-run` (`runId` + `startedAt`) + duration/plausibility/rate-limit validation in `/api/submit-run` | M1 | Explorer 2 (P0) |
| 5 | Rich `/api/init` & Social Redis Hooks | Return `username`, `streak`, `personalBest`, `myRank`, `totalPlayers`, `bossKillsToday`, `siegeTarget`, `leaderboard`, `topGhost`, `mutatorOverride`, `mutatorVotes` | M1 | Explorer 1 & 2 (P0/P1) |
| 6 | Winner-Picks-Tomorrow's-Mutator | `POST /api/vote-mutator` (top players vote tomorrow's `DailyModifier`) + apply in `/api/init` & post title | M1 | Explorer 2 (P1) |
| 7 | Wordle-Style Emoji Build Share Card | Rich `/api/share-comment` with rank, score, streak, mutator, weapon emoji strip, and client error feedback | M1 | Explorer 1 & 2 (P1) |
| 8 | Mobile WebAudio Unlock + Pitched SFX + Drone | Sync `pointerdown`/`touchstart` `AudioContext` unlock, `emberfall_play_sfx_pitched(kind, pitch)`, toggleable dark-fantasy bass drone, mute state | M1 | Explorer 3 (P0) |
| 9 | TypeScript & WASM Toolchain Setup | Add `tsconfig.json`, install `wasm32-unknown-unknown` target (and `wasm-opt` if available), verify `build:server` & `build:wasm` | M1 & M3 | Explorer 1 & 3 (P0) |
| 10 | Wire `/api/init` & Share/Submit Callbacks into Rust | `emberfall_bridge` passes init JSON & async submit/share status to Rust (`Title`, `HUD`, `GameOver`/`Victory` leaderboard + streak + siege bar) | M2 | Explorer 1 & 2 (P0) |
| 11 | #1 Champion Ghost Replay | Record 2 Hz `(x, y)` path (180 samples, ~1.4 KB), submit on PB/#1, render translucent golden `u/Champion` phantom in-game | M2 | Explorer 2 (P1) |
| 12 | Hitstop + Trauma Shake + Squash & Stretch | `hitstop` freeze frames (35–80ms), quadratic `trauma * trauma` camera shake, sprite squash/stretch, inner-fill `TelegraphRing` | M2 | Explorer 3 (P0) |
| 13 | GLSL100 Hit-Flash & Post-Processing Shaders | `shaders.rs`: `#version 100` `precision mediump float;` white hit-flash material + `render_target` vignette/low-HP pulse/chromatic aberration/shockwave | M2 | Explorer 3 (P1) |
| 14 | 90s Survivor Depth: Evolutions, Blood Shrines & Fever | 3 Weapon Evolutions (Lv.3+ synergy), 1–2 deterministic Blood Shrines (2s channel -> elite + `+0.5x` mult), Kill-Streak Fever Meter (`x1.0`–`x2.5`) | M2 | Explorer 2 (P2) |
| 15 | Pause & Mute Controls + Combat Clarity | Touch/keyboard Pause (`P`/`Esc`/HUD button) & Audio Mute toggle, XP combo pitch-escalation | M2 | Explorer 1 & 3 (P0) |
| 16 | Full WASM + Server Integration & Whole-Project Gate | Rebuild `emberfall.wasm` (`wasm-opt -Oz --all-features`), `dist/server/index.cjs`, verify CSP (no external URLs), run `cargo test` + `tsc` + whole-project Auditor | M3 | Explorer 1, 2, 3 (P0) |

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| M1 | Devvit Compliance, Server & Splash/Audio Bridge | `tsconfig.json`, `devvit.json`, `src/server/index.ts`, `dist/client/splash.html`, `dist/client/index.html`, `package.json`, `wasm32` target check | none | DONE |
| M2 | Pure-Rust Juice, Shaders, Ghost Replay & 90s Depth | `crates/emberfall/src/shaders.rs`, `rng.rs`, `weapons.rs`, `main.rs` | Interface Contracts (defined below) | DONE |
| M3 | Final Integration, WASM Build & Whole-Project Audit | Full `cargo test` + `wasm32` build + `npm run build` + CSP check + Whole-Project Auditor | M1, M2 | DONE |

## Interface Contracts

### 1. HTTP API (`src/server/index.ts` <-> `dist/client/splash.html` & `dist/client/index.html`)
- `GET /api/init`:
  Returns JSON:
  ```json
  {
    "seed": "20261121",
    "mutatorOverride": 0, // -1 if none, else 0..3 matching DailyModifier index
    "username": "redditor_or_null",
    "streak": 3,
    "personalBest": 8450,
    "myRank": 4, // 0 if unranked
    "totalPlayers": 42,
    "bossKillsToday": 18,
    "siegeTarget": 25,
    "leaderboard": [{ "username": "alice", "score": 14200, "rank": "SSS" }],
    "topGhost": { "username": "alice", "score": 14200, "points": [[0,0],[12,-8]] }, // null if none
    "canVoteMutator": true // true if user is in top 3 or #1
  }
  ```
- `POST /api/start-run`:
  Returns `{ "ok": true, "runId": "<uuid_or_hex>", "startedAt": 1763737800000 }`.
- `POST /api/submit-run`:
  Body: `{ "runId": "...", "score": number, "kills": number, "bossSlain": boolean, "level": number, "buildSummary": string, "ghostPoints": [[x,y], ...] }`
  Validates `runId` (if present, elapsed `10s..180s`), rate-limits per user (`rate:<user>`), clamps bounds, updates `ef:lb`, `ef:meta`, `ef:streak`, `ef:bosses`, and `ef:ghost` if new #1.
  Returns `{ "ok": true, "rank": "S", "myRank": 2, "totalPlayers": 43, "streak": 4, "bossKillsToday": 19, "isNewBest": true, "canVoteMutator": true }`.
- `POST /api/share-comment`:
  Body: `{ "score": number, "kills": number, "bossSlain": boolean, "level": number, "streak": number, "mutatorName": string, "buildEmojis": string, "rank": string, "myRank": number }`
  Posts Wordle-style Markdown + emoji build card as `USER`. Returns `{ "ok": true }` or `401`/`429` JSON `{ "ok": false, "error": "..." }`.
- `POST /api/vote-mutator`:
  Body: `{ "mutatorIndex": 0..3 }`
  Stores tomorrow's mutator vote in `ef:mutator-vote:{sub}:{tomorrowSeed}` (if user is eligible / top 3). Returns `{ "ok": true, "mutatorIndex": number }`.

### 2. Rust <-> JS `miniquad_add_plugin` FFI (`crates/emberfall/src/main.rs` <-> `dist/client/index.html`)
All pointers passed from JS -> Rust or Rust -> JS must be safe across WASM linear memory:
- **Existing + Upgraded Rust imports (`extern "C"` from `"env"`)**:
  - `emberfall_get_day_number() -> u32` (returns UTC `YYYYMMDD`)
  - `emberfall_play_sfx_pitched(kind: i32, pitch_mul: f32)` (keep `emberfall_play_sfx(kind: i32)` as alias or replace cleanly in both `index.html` and `main.rs`)
  - `emberfall_set_muted(muted: i32)` (`1` = mute, `0` = unmute)
  - `emberfall_toggle_drone(active: i32)` (`1` = play/resume subtle dark drone during `Playing`, `0` = pause/fade)
  - `emberfall_on_run_start()` (triggers `POST /api/start-run` in JS, caches `runId`)
  - `emberfall_on_run_complete(score: u32, kills: u32, boss_slain: i32, level: u32, build_ptr: *const u8, build_len: u32, ghost_ptr: *const i16, ghost_count: u32)` (ghost_count = number of `(x,y)` i16 pairs, max 200; JS reads `memory.buffer` synchronously before async `fetch`)
  - `emberfall_share_comment(score: u32, kills: u32, boss_slain: i32, level: u32, build_ptr: *const u8, build_len: u32)`
  - `emberfall_vote_mutator(mutator_idx: i32)`
- **JS -> Rust Pull Polling (Zero-Alloc / Safe Buffer FFI — simplest & 100% crash-proof in `miniquad`)**:
  Instead of JS calling into Rust pointers asynchronously, Rust polls synchronous getters each frame (or when `status_dirty` changes):
  - `emberfall_poll_init_ready() -> i32` (`1` once `/api/init` resolved, `0` pending)
  - `emberfall_get_init_field(field_id: i32) -> i32` (`0`: `streak`, `1`: `personalBest`, `2`: `myRank`, `3`: `totalPlayers`, `4`: `bossKillsToday`, `5`: `siegeTarget`, `6`: `mutatorOverride` (`-1` if none), `7`: `canVoteMutator` (`0/1`), `8`: `leaderboard_count`, `9`: `ghost_point_count`)
  - `emberfall_copy_str_field(field_id: i32, index: i32, out_ptr: *mut u8, max_len: u32) -> u32` (`field_id=0`: current `username`, `1`: `leaderboard[index].username`, `2`: `ghost_username`, `3`: `last_status_msg`)
  - `emberfall_get_lb_score(index: i32) -> u32`
  - `emberfall_get_ghost_xy(index: i32) -> i32` (packed `(x as i16) << 16 | (y as u16)` or two getters `emberfall_get_ghost_x(idx) -> i32`, `emberfall_get_ghost_y(idx) -> i32` — two separate `i32` getters is even simpler and zero-bitmask-bug!)
  - `emberfall_get_share_state() -> i32` (`0`: idle, `1`: pending, `2`: ok, `3`: error)
  - `emberfall_get_submit_state() -> i32` (`0`: idle, `1`: pending, `2`: ok, `3`: error)
  *(Note: On non-wasm32 native builds, stub all `extern "C"` functions cleanly so `cargo check` and `cargo test` work natively!)*
