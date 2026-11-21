# EMBERFALL: DAILY HORDE

**EMBERFALL: DAILY HORDE** is a 90-second Dark Fantasy Action Horde Survivor ARPG written in **100% pure Rust** using [`not-fl3/macroquad`](https://github.com/not-fl3/macroquad) compiled to **WebAssembly (`.wasm`)**, built for **Reddit's Game of the Day Hackathon 2026** on **Devvit Web** and **Redis**.

---

## Architecture (Devvit Web + Redis + Pure Rust)

- **100% Pure Rust WASM Client (`crates/emberfall/` → `dist/client/emberfall.wasm`)**: All game loop logic, AI hordes, GLSL100 shaders, particle explosions, audio triggers, and in-game HUD/menus are compiled from Rust with zero HTML/JS/CSS game logic and all PNGs embedded via `include_bytes!()`.
- **Devvit Web Entrypoints (`dist/client/`)**:
  - `splash.html` (`default` inline feed card, `<30 KB`, zero WASM): Displays today's UTC Daily Mutator, Subreddit Boss Siege progress bar, and Top 3 Champions, then launches fullscreen via `requestExpandedMode(event, 'game')`.
  - `index.html` (`game` expanded fullscreen host): Loads `mq_js_bundle.js` + `emberfall.wasm` and registers the `emberfall_bridge` plugin for WebAudio SFX/drone, UTC day/init state, run lifecycle, ghost replay, mutator voting, and Reddit comment sharing.
- **Devvit Web + Redis Backend (`src/server/index.ts` → `dist/server/index.cjs`)**:
  - **UTC Daily Scheduler**: Automated daily post creation via `scheduler.tasks` + atomic Redis `SET NX` lock.
  - **Privacy & Retention**: 30-day (`30 * 86400`s) TTLs on all daily keys plus `onPostDelete` trigger cleanup.
  - **Run Session Anti-Cheat**: `POST /api/start-run` issues a `runId` + `startedAt`, validated on `POST /api/submit-run` (10s–180s duration bounds, score/kill clamping, per-user rate limit).
  - **Community & Social Hooks**:
    - **#1 Champion Ghost Replay**: Records 2 Hz `(x, y)` hero path (~1.4 KB) and renders the daily `#1` champion as a translucent golden phantom in-game.
    - **Subreddit Boss Siege**: Collective daily dragon-slain progress bar across the subreddit.
    - **Winner-Picks-Tomorrow's-Mutator**: Top 3 daily players vote on tomorrow's `DailyModifier` (`POST /api/vote-mutator`).
    - **Wordle-Style Emoji Build Share Card**: 1-tap `POST /api/share-comment` posts rank, score, streak, mutator, and weapon emoji strip directly to the Reddit thread.

---

## Pure-Rust Gameplay Features

- **GLSL100 Shaders (`crates/emberfall/src/shaders.rs`)**: `#version 100` hit-flash sprite material and `render_target` post-processing (vignette, low-HP pulse, chromatic aberration, and dash/evolution shockwave).
- **Tactile Combat Juice**: Hitstop freeze frames (35–80ms), quadratic `trauma * trauma` camera shake, sprite squash/stretch, and inner-fill telegraph rings.
- **3 Weapon Evolutions (`crates/emberfall/src/weapons.rs`)**: Max-level weapon + passive synergies unlock *Meteor Barrage*, *Crimson Reaper*, and *Inferno Vortex*.
- **Blood Shrines & Kill-Streak Fever Meter**: 1–2 deterministic Blood Shrines per daily seed (2s channel spawns an elite for `+0.5x` score multiplier) + Fever Meter (`x1.0` to `x2.5` multiplier).
- **1-Thumb Mobile Controls + Pause/Mute**: Virtual thumbstick anywhere on screen (excluding top-right Pause/Mute HUD buttons), bottom-right **DASH**, and `P` / `Esc` / `M` keyboard shortcuts.

---

## Controls

| Action | Mobile / Touch | Desktop |
| --- | --- | --- |
| **Move Hero** | Drag anywhere (1-thumb virtual joystick) | `WASD` or Arrow Keys |
| **Auto-Attack** | Automatic (targets nearest enemies) | Automatic |
| **Inferno Dash** | Tap **DASH** button (bottom-right) | `Space` or Right-Click |
| **Pick Level-Up Card** | Tap card | Click card or press `1` / `2` / `3` |
| **Pause / Mute** | Tap top-right `||` / `S` HUD buttons | `P` or `Esc` (Pause), `M` (Mute) |

---

## Build, Test & Deployment

```bash
# 1. Install Devvit Web server dependencies
npm install

# 2. Check & test the Rust crate (0 warnings, deterministic 365-day & shrine tests)
cargo check --manifest-path crates/emberfall/Cargo.toml
cargo test --manifest-path crates/emberfall/Cargo.toml

# 3. Typecheck & build the Devvit Web server
npm run typecheck
npm run build:server

# 4. Build Rust -> WebAssembly (dist/client/emberfall.wasm)
npm run build:wasm

# 5. Playtest & Upload to Reddit Developers
npx devvit login
npx devvit playtest <your-public-test-subreddit>
npx devvit upload
```

---

## Open-Source Credits & Licenses

- **Game Engine:** [`not-fl3/macroquad`](https://github.com/not-fl3/macroquad) — Dual-licensed under MIT / Apache-2.0.
- **VFX Particle Textures:** [`Calinou/kenney-particle-pack`](https://github.com/Calinou/kenney-particle-pack) (by Kenney) — Licensed under **CC0 1.0 Universal (Public Domain)**.
- **Platform SDK:** [`@devvit/web`](https://developers.reddit.com/) — Reddit Developer Platform.

