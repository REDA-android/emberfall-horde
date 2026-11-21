# Codebase Inspection Handoff — EMBERFALL: DAILY HORDE

**Core Finding:** `emberfall-horde` is a compact, working 2,512-line pure-Rust (`macroquad 0.4` -> `wasm32-unknown-unknown`, 931 KB) 90-second Horde Survivor ARPG paired with a `@devvit/web` + Redis server (`src/server/index.ts`, 246 lines). While the core gameplay loop, deterministic UTC daily seed, sprites/VFX, and score submission work, **the client never calls `/api/init` or surfaces leaderboards/streaks/community boss kills**, lacks audio mute/pause/shaders/custom fonts/meta-progression, and the VM currently lacks the `wasm32-unknown-unknown` Rust target.

---

## 1. Observation

### 1.1 Repository Structure & Git History
- **Git log (`git log -n 20 --oneline --decorate`):** Single commit on `main`:
  - `80f9819 (HEAD -> main, origin/main) Build EMBERFALL: DAILY HORDE — 100% Rust/WASM ARPG with real sprites, Kenney CC0 VFX, and Devvit Web + Redis`
- **Root files:**
  - `README.md` (53 lines): Documents features, controls, Devvit Web build/upload steps, and credits (`not-fl3/macroquad`, `Calinou/kenney-particle-pack`, `@devvit/web`).
  - `package.json` (23 lines): `"type": "module"`, `@devvit/web: ^0.15.4`, `devvit: ^0.15.4`, `typescript: ^5.7.2`, `vite: ^5.4.21`, `@types/node: ^22.10.0`. Scripts: `build:wasm`, `build:server`, `build`, `devvit:playtest`, `devvit:upload`.
  - `devvit.json` (43 lines): Devvit schema v1, `post.dir: "dist/client"`, `post.entrypoints.default.entry: "index.html"`, `height: "tall"`, `server.dir: "dist/server"`, `entry: "index.cjs"`. Permissions: `redis: true`, `reddit: { enable: true, asUser: ["SUBMIT_COMMENT"] }`. Trigger: `onAppInstall -> /internal/on-app-install`. Mod menu: `/internal/menu/post-create`.
  - `vercel.json` (5 lines): `buildCommand: "echo 'Using prebuilt Rust WASM in dist/client'"`, `outputDirectory: "dist/client"`.
  - `vite.server.config.ts` (22 lines): SSR CJS bundle of `src/server/index.ts` -> `dist/server/index.cjs` (`target: 'node22'`, `noExternal: true`, `inlineDynamicImports: true`).
  - **Missing config:** No `tsconfig.json` in project root.

### 1.2 Rust Client (`crates/emberfall/`)
- **`crates/emberfall/Cargo.toml` (16 lines):** Package `emberfall v1.0.0`, `edition = "2021"`, single dependency `macroquad = "0.4"`. Release profile: `opt-level = "z"`, `lto = true`, `codegen-units = 1`, `panic = "abort"`, `strip = true`.
- **`crates/emberfall/src/assets.rs` (46 lines):** `GameAssets::load()` embeds all 13 PNGs at compile time via `include_bytes!()` (`FilterMode::Linear`), making startup instant with zero runtime network asset fetches.
- **`crates/emberfall/src/rng.rs` (133 lines):**
  - `Rng`: FNV-1a 64-bit seed hash + `xorshift64*` PRNG (`from_seed_str`, `next_u64`, `next_f32`, `range_f32`, `range_usize`).
  - `DailyModifier` (4 variants): `BloodMoon` (+25% crit, faster hordes), `ArcaneSurge` (+20% attack speed, 2x XP gems), `SwarmRush` (massive swarms, +30% score), `TitansWrath` (enraged boss 2100 HP vs 1650 HP, +50% boss bounty).
  - `compute_rank(score, boss_slain)`: `E` (<500), `D` (500+), `C` (1200+), `B` (2500+), `A` (4500+), `S` (7000+), `SS` (9500+ & boss slain), `SSS` (14000+ & boss slain).
  - Unit test `rng::tests::deterministic_365_days_and_ranks` (lines 108-131).
- **`crates/emberfall/src/weapons.rs` (151 lines):**
  - `UpgradeId` (8 upgrades, max level 5 each): `ArcaneSlash`, `EmberBolts`, `ChainLightning`, `RunicOrbit`, `NovaDash`, `OverclockHaste`, `SoulMagnet`, `VampiricWard`.
  - `HeroBuild`: Starts at `slash_lvl: 1, bolt_lvl: 1, dash_lvl: 1`, others `0`. `roll_three(&mut Rng)` uses Fisher-Yates shuffle over upgrades `< 5` (falls back to `ALL_UPGRADES` if `< 3` remain).
- **`crates/emberfall/src/main.rs` (1,864 lines):**
  - **FFI Bridge (lines 10-34):** `extern "C"` imports from `"env"`: `emberfall_get_day_number() -> u32`, `emberfall_on_run_complete(score, kills, boss_slain, level)`, `emberfall_share_comment(score, kills, boss_slain)`, `emberfall_play_sfx(kind)`. Non-wasm32 stubs return `20261121` and no-ops.
  - **Constants (lines 36-38):** `ARENA_HALF = 1050.0` (2100x2100 world), `RUN_DURATION = 90.0`s, `BOSS_SPAWN_TIME = 55.0`s.
  - **State Machine (`Phase`, lines 41-47):** `Title`, `Playing`, `LevelUp`, `GameOver`, `Victory`.
  - **Entities:** `Enemy` (`Imp`, `Golem`, `Wraith`, `Boss`), `Projectile`, `SlashArc`, `LightningArc`, `TelegraphRing` (boss AoE slam), `XpGem`, `Particle` (`VfxKind` — `Light` variant has `#[allow(dead_code)]` at line 114), `FloatText`, `Afterimage`.
  - **Controls (lines 621-700):** `WASD`/Arrows + `Space`/Right-Click for Dash on desktop; 1-thumb virtual joystick anywhere on screen + bottom-right `DASH` circle button (`dash_center = (sw - 68, sh - 76)`, `dash_r = 44`) for touch/mouse. Level-up cards support tap/click or `1`/`2`/`3` keys (lines 1749-1752).
  - **Gameplay Loop (`update_playing`, lines 702-1177):**
    - 90s countdown; surviving to 90s triggers `Phase::Victory` + `+1500` score bonus (lines 717-723).
    - Enemy cap `< 75` (line 772), soft separation loop over next 10 neighbors (lines 1000-1013).
    - 4 auto-weapons: Arcane Slash (line 802), Ember Bolts (line 835), Chain Lightning (line 863), Runic Orbit Blades (line 904), plus active Inferno Dash nova (line 442).
    - Boss (`Ember Lich Overlord`) spawns at 55s with 8-way spiral projectiles + telegraphed slam ring (lines 952-982).
  - **Rendering (`draw_world`, lines 1207-1512; `draw_ui`, lines 1542-1834):**
    - `Camera2D` with screen shake + smooth lerp.
    - Tiled 256x256 `arena_floor.png`, glowing runic border, ground shadows (`draw_ellipse`), sprite bobbing, white flash + `vfx_light` overlay on hit, Kenney particles, floating crit/damage text.
    - No custom GLSL shaders; uses macroquad's default 2D renderer and built-in bitmap font (`draw_text` / `measure_text`).

### 1.3 HTML/JS Shell & Web Audio (`dist/client/`)
- **`dist/client/index.html` (75 lines):**
  - Loads `./mq_js_bundle.js` (37 KB, miniquad v2 JS runtime) and `./emberfall.wasm` (931 KB).
  - Registers `emberfall_bridge` plugin via `miniquad_add_plugin` (lines 34-68):
    - `emberfall_get_day_number`: returns `YYYYMMDD` in UTC.
    - `emberfall_on_run_complete`: `fetch('/api/submit-run', POST)` — **errors silently swallowed** (`.catch(function () {})`, line 45).
    - `emberfall_share_comment`: `fetch('/api/share-comment', POST)` — **errors silently swallowed** (line 52).
    - `emberfall_play_sfx(kind)`: Web Audio API `AudioContext` oscillator synth for 9 SFX kinds (`0..=8`, lines 16-32, 54-64).

### 1.4 Devvit Web + Redis Server (`src/server/index.ts`, 247 lines)
- **Redis Key Schema (lines 63-67):**
  - `ef:lb:{sub}:{seed}` — Sorted set (leaderboard scores by username).
  - `ef:meta:{sub}:{seed}:{user}` — JSON `StoredRunMeta` (`score`, `kills`, `bossSlain`, `level`, `rank`).
  - `ef:streak:{sub}:{user}` — JSON `{ count, lastDate }`.
  - `ef:bosses:{sub}:{seed}` — Integer counter of daily boss kills in subreddit.
  - `ef:shared:{postId}:{seed}:{user}` — Highest score shared as comment (dedup guard).
- **Endpoints:**
  - `POST /internal/on-app-install` (line 87): Calls `createDailyPost()`.
  - `POST /internal/menu/post-create` (line 93): Mod menu action creating daily custom post.
  - `GET /api/init` (line 102): Fetches `reddit.getCurrentUsername()`, top-10 `redis.zRange`, and `bossKillsToday`.
  - `POST /api/submit-run` (line 127): Clamps `score <= 500000`, `kills <= 10000`, `level <= 99`; computes rank; increments `ef:bosses` if `bossSlain`; updates `ef:lb` & `ef:meta` if new PB; updates `ef:streak`; returns `{ ok: true, rank }`.
  - `POST /api/share-comment` (line 188): Requires `context.postId` and authenticated `username` (401 otherwise); checks `shareGuardKey`; calls `reddit.submitComment({ id: context.postId, text, runAs: 'USER' })`.
- **CSP / Headers:** `sendJson` sets `Content-Type: application/json; charset=utf-8` and `Cache-Control: no-store`. No explicit CSP headers (managed by Devvit Web / Vercel host).

### 1.5 Assets & Bundle Sizes
- **Sprites (`crates/emberfall/assets/sprites/`, 356 KB total):**
  - `arena_floor.png` (100K, 256x256 RGB)
  - `boss_lich.png` (92K, 256x256 RGBA)
  - `enemy_golem.png` (49K, 160x160 RGBA)
  - `enemy_wraith.png` (35K, 144x144 RGBA)
  - `enemy_imp.png` (34K, 128x128 RGBA)
  - `hero_knight.png` (38K, 160x160 RGBA)
- **VFX (`crates/emberfall/assets/vfx/`, 96 KB total, all 128x128 RGBA):**
  - `fire_01.png` (18K), `flame_01.png` (9.7K), `light_01.png` (16K), `magic_05.png` (12K), `slash_01.png` (4.8K), `spark_01.png` (17K), `star_07.png` (6.0K).
- **Fonts / Audio Files:** None on disk.
- **Built Artifacts:** `dist/client/emberfall.wasm` (931 KB), `dist/client/mq_js_bundle.js` (37 KB), `dist/client/index.html` (3.4 KB), `dist/server/index.cjs` (3.0 MB / 3,060.10 kB).

---

## 2. Logic Chain — Key Gaps, Bugs & Limitations

1. **Unused `/api/init` & Invisible Social/Redis Features:** `src/server/index.ts:102-125` implements `GET /api/init` (top-10 leaderboard, username, `bossKillsToday`) and `ef:streak` tracking (lines 167-182), yet `dist/client/index.html` and `crates/emberfall/src/main.rs` never call `/api/init` and have no FFI callback to receive leaderboard/streak/community data. Result: players cannot see the Subreddit Leaderboard, their Reddit username, their daily streak, or community boss kills in-game.
2. **Optimistic Share & Silent Network Failures:** `main.rs:1812-1819` sets `self.shared_comment = true` unconditionally on button press, while `index.html:45,52` swallows fetch errors/401s. On Vercel (static host) or for unauthenticated Reddit Guests, clicking "POST SCORE TO REDDIT" falsely shows `"SHARED TO REDDIT ✓"`.
3. **No Automated Daily Post Scheduler:** `devvit.json:24-37` only creates posts on app install or manual moderator menu click; there is no Devvit scheduler cron job to auto-post daily at 00:00 UTC.
4. **No Client-Side Meta-Progression or Run Replayability Depth:** `HeroBuild` resets every run; no persistent unlocks, hero classes, weapon evolutions/synergies, or local/Redis meta-upgrades exist. Single boss (`Boss`), 3 regular enemies, 1 arena.
5. **Rendering/UX/Audio Polish Limits:** No custom GLSL shaders (bloom/vignette/hit-flash), no custom TTF font (uses macroquad default bitmap font), no pause button during `Phase::Playing`, no audio mute toggle, no background music/ambient track.
6. **Build Environment Gaps:** `wasm32-unknown-unknown` target is not installed on this VM (blocking `npm run build:wasm`), `wasm-opt` is not installed, and there is no `tsconfig.json` (causing bare `npx tsc --noEmit src/server/index.ts` to fail on `@devvit/*` `.d.ts` files).
7. **Server Validation:** `/api/submit-run` (`src/server/index.ts:141-145`) trusts client-supplied `score`, `kills`, `bossSlain`, `level` with only static upper-bound clamps.

---

## 3. Caveats

- `dist/client/emberfall.wasm` (931 KB) is prebuilt and committed; because `wasm32-unknown-unknown` is not installed on this VM, we verified native Rust compilation (`cargo check`, `cargo test`) rather than rebuilding the `.wasm` binary from scratch.
- Live Reddit Devvit runtime behavior (`reddit.getCurrentUsername()`, `reddit.submitComment()`, Redis persistence) was inspected via source code rather than executed against a live subreddit.

---

## 4. Conclusion

The codebase is clean, self-contained, and builds/tests cleanly on native Rust (`cargo check`, `cargo test`) and Vite SSR (`npm run build:server`). Its highest-leverage gaps for winning **Reddit's Game of the Day Hackathon 2026** are:
1. Wiring `/api/init` (leaderboard, daily streak, `bossKillsToday`, username) into the Rust/WASM UI and fixing share/submit error feedback.
2. Adding daily post scheduling (`devvit.json` scheduler), weapon evolutions/synergies, multiple heroes/modifiers/boss phases, custom font + GLSL shaders, pause/mute controls, and Web Audio music/SFX polish.
3. Adding `tsconfig.json` and installing `wasm32-unknown-unknown` (+ `wasm-opt`) for full local WASM builds.

---

## 5. Verification Method

All checks below were executed in `/home/antigravity/REDA-android/emberfall-horde`:

| Check | Command | Result |
|---|---|---|
| Toolchain | `rustc --version; cargo --version; node -v; npm -v` | `rustc 1.100.0`, `cargo 1.100.0`, `node v24.22.3`, `npm 11.21.2` |
| Rust check | `cargo check --manifest-path crates/emberfall/Cargo.toml` | **PASS** (exit 0, 0 warnings) |
| Rust tests | `cargo test --manifest-path crates/emberfall/Cargo.toml` | **PASS** (`1 passed; 0 failed` — `deterministic_365_days_and_ranks`) |
| WASM target | `cargo check --manifest-path crates/emberfall/Cargo.toml --target wasm32-unknown-unknown` | **FAIL** (`E0463: can't find crate for std` — target not installed) |
| Server TS (bare) | `npx tsc --noEmit src/server/index.ts` | **FAIL** (missing `tsconfig.json`; errors in `node_modules/@devvit/*`) |
| Server TS (ES2022) | `npx tsc --noEmit --target es2022 --module nodenext --moduleResolution nodenext --skipLibCheck src/server/index.ts` | **PASS** (exit 0, 0 errors) |
| Server bundle | `npm run build:server` | **PASS** (`dist/server/index.cjs` 3,060.10 kB in 4.62s) |
