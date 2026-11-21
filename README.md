# EMBERFALL: DAILY HORDE

**EMBERFALL: DAILY HORDE** is a 90-second Dark Fantasy Action Horde Survivor ARPG written in **100% pure Rust** using [`not-fl3/macroquad`](https://github.com/not-fl3/macroquad) compiled to **WebAssembly (`.wasm`)**, built for **Reddit's Game of the Day Hackathon 2026** on **Devvit Web** and **Redis**.

---

## Architecture (Devvit Web + Redis + Pure Rust)

- **100% Pure Rust WASM Client (`crates/emberfall/` → `dist/client/emberfall.wasm`)**: All game loop logic, 3 playable Hero Classes, 3 Daily Biomes, Runic Explosive Barrels, AI hordes, GLSL100 shaders, particle explosions, audio triggers, tactical Minimap Radar, and in-game HUD/menus are compiled from Rust with zero HTML/JS/CSS game logic and all PNGs + Open-Source TTF typography embedded via `include_bytes!()`.
- **Devvit Web Entrypoints (`dist/client/`)**:
  - `splash.html` (`default` inline feed card, `<30 KB`, zero WASM): Displays today's UTC Daily Biome & Mutator, Subreddit Faction War & Boss Siege progress bar, and Top 3 Champions, then launches fullscreen via `requestExpandedMode(event, 'game')`.
  - `index.html` (`game` expanded fullscreen host): Loads `mq_js_bundle.js` + `emberfall.wasm` and registers the `emberfall_bridge` plugin for WebAudio SFX/drone, UTC day/init state, run lifecycle, ghost replay, faction war sync, mutator voting, and Reddit comment sharing.
- **Devvit Web + Redis Backend (`src/server/index.ts` → `dist/server/index.cjs`)**:
  - **UTC Daily Scheduler & Yesterday's Champion Coronation**: Automated daily post creation via `scheduler.tasks` + atomic Redis `SET NX` lock, automatically posting an official comment crowning yesterday's `#1 Champion` and winning Reddit Faction Order.
  - **Privacy & Retention**: 30-day (`30 * 86400`s) TTLs on all daily keys plus `onPostDelete` trigger cleanup.
  - **Run Session Anti-Cheat**: `POST /api/start-run` issues a `runId` + `startedAt`, validated on `POST /api/submit-run` (10s–180s duration bounds, score/kill clamping, per-user rate limit).
  - **Community & Social Hooks**:
    - **3-Faction Daily Subreddit War**: Every run contributes points to the player's chosen Order (`☀️ Solar Order`, `🔮 Arcane Conclave`, or `🗡️ Shadow Guild`) in Redis (`ef:factions:{sub}:{seed}`).
    - **#1 Champion Ghost Replay**: Records 2 Hz `(x, y)` hero path (~1.4 KB) and renders the daily `#1` champion as a translucent golden phantom in-game and on the Minimap Radar.
    - **Subreddit Boss Siege**: Collective daily Arch-Lich slain progress bar across the subreddit.
    - **Winner-Picks-Tomorrow's-Mutator**: Top 3 daily players vote on tomorrow's `DailyModifier` (`POST /api/vote-mutator`).
    - **Wordle-Style Emoji Build Share Card**: 1-tap `POST /api/share-comment` posts rank, Hero Class, Faction Order, Daily Biome, score, streak, mutator, and weapon emoji strip directly to the Reddit thread (zero external links).

---

## Pure-Rust Gameplay Features

- **3 Playable Hero Classes & Illustrated Sprites (`crates/emberfall/src/weapons.rs`)**:
  - **☀️ Sun Knight (*Solar Order*)**: `130 HP`, balanced melee + fireball starter (`Arcane Slash Lv.1` + `Ember Bolts Lv.1`).
  - **🔮 Pyromancer (*Arcane Conclave*)**: `95 HP`, long-range spellcaster (`Ember Bolts Lv.1` + `Chain Lightning Lv.1` + `Soul Magnet Lv.1`).
  - **🗡️ Shadow Assassin (*Shadow Guild*)**: `105 HP`, high-mobility rogue (`Arcane Slash Lv.1` + `Runic Orbit Lv.1` + `Inferno Dash Lv.2` + `Arcane Haste Lv.1`).
- **3 Deterministic Daily Biomes & Runic Explosive Barrels (`crates/emberfall/src/rng.rs`)**:
  - **Obsidian Sanctuary**: Arcane Barrels (`+25%` shockwave damage).
  - **Volcanic Rift**: Magma Barrels (`+30%` explosion radius).
  - **Frost Abyss**: Cryo Barrels (chills & slows hit enemies by `42%` for `3.5s`).
  - Barrels chain-detonate nearby barrels when destroyed by attacks or Dash!
- **Arch-Lich Phase II Enrage & SUPERNOVA Ultimate**:
  - At `<50% HP`, the Daily Boss enters **Phase II Enrage** with double telegraphed slams and 12-way crimson spiral barrages.
  - Slaying enemies charges the **SUPERNOVA (`NOVA`)** ultimate ability for a screen-clearing triple shockwave.
- **Tactical Minimap Radar & Embedded TTF Typography**:
  - Real-time top-right radar tracking the Hero, Blood Shrines, Runic Barrels, `#1 Champion Ghost`, and Boss.
  - Embedded `ChakraPetch-Bold.ttf` (SIL Open Font License 1.1) rendered via `load_ttf_font_from_bytes` with drop shadows.
- **GLSL100 Shaders (`crates/emberfall/src/shaders.rs`)**: `#version 100` hit-flash sprite material and `render_target` post-processing (biome-tinted vignette, low-HP pulse, chromatic aberration, and shockwave distortion).
- **3 Weapon Evolutions (`crates/emberfall/src/weapons.rs`)**: Max-level weapon + passive synergies unlock *Meteor Barrage*, *Crimson Reaper*, and *Inferno Vortex*.
- **Blood Shrines & Kill-Streak Fever Meter**: 1–2 deterministic Blood Shrines per daily seed (2s channel spawns an elite for `+0.5x` score multiplier) + Fever Meter (`x1.0` to `x2.5` multiplier).

---

## Controls

| Action | Mobile / Touch | Desktop |
| --- | --- | --- |
| **Select Hero Class** | Tap Class Card on Title Screen | Click Card or press `1` / `2` / `3` |
| **Move Hero** | Drag anywhere (1-thumb virtual joystick) | `WASD` or Arrow Keys |
| **Auto-Attack** | Automatic (targets nearest enemies & barrels) | Automatic |
| **Inferno Dash** | Tap **DASH** button (bottom-right) | `Space` or Right-Click |
| **Supernova Ultimate** | Tap **NOVA** button (left of DASH when charged) | `Q` or `E` |
| **Pick Level-Up Card** | Tap card | Click card or press `1` / `2` / `3` |
| **Pause / Mute** | Tap top-right `||` / `S` HUD buttons | `P` or `Esc` (Pause), `M` (Mute) |

---

## Build, Test & Deployment

```bash
# 1. Install Devvit Web server dependencies
npm install

# 2. Check & test the Rust crate (0 warnings, deterministic 365-day, biome, barrel & class tests)
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
- **Embedded TTF Typography:** [`Chakra Petch Bold`](https://github.com/google/fonts/tree/main/ofl/chakrapetch) (Cadson Demak / Google Fonts) — Licensed under **SIL Open Font License 1.1 (OFL-1.1)**.
- **Platform SDK:** [`@devvit/web`](https://developers.reddit.com/) — Reddit Developer Platform.

