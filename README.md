# EMBERFALL: DAILY HORDE

**EMBERFALL: DAILY HORDE** is a 90-second Dark Fantasy Action Horde Survivor ARPG written in **100% pure Rust** using [`not-fl3/macroquad`](https://github.com/not-fl3/macroquad) compiled to **WebAssembly (`.wasm`)**, built for **Reddit's Game of the Day Hackathon 2026** on **Devvit Web** and **Redis**.

---

## Architecture (Devvit Web + Redis + Pure Rust)

- **100% Pure Rust WASM Client (`crates/emberfall/` → `dist/client/emberfall.wasm`)**: All game loop logic, 3 playable Hero Classes, 3 Daily Biomes, Runic Explosive Barrels, AI hordes, GLSL100 shaders, particle explosions, audio triggers, tactical Minimap Radar, and in-game HUD/menus are compiled from Rust with zero HTML/JS/CSS game logic and all PNGs + Open-Source TTF typography embedded via `include_bytes!()`.
- **Devvit Web Entrypoints (`dist/client/`)**:
  - `splash.html` + `splash.js` (`default` inline feed card, `<30 KB`, zero WASM, zero inline scripts): Dynamically adapts its color palette to today's UTC Daily Biome (*Obsidian* purple/gold, *Volcanic* lava/ember, *Frost* ice/cyan), shows today's Boss, Mutator, Community Faction Blessing, 3 Daily Bounties, live 3-segment Faction War percentage bar, Subreddit Boss Siege progress & live Siege Activity Feed, Top 3 Champions, Tomorrow's Dungeon Countdown & Vote Leader teaser, plus an interactive `? Guide` briefing drawer before launching fullscreen via `requestExpandedMode(event, 'game')`.
  - `index.html` + `game_bridge.js` (`game` expanded fullscreen host, zero inline scripts): Loads `mq_js_bundle.js` + `emberfall.wasm` and registers the `emberfall_bridge` plugin for layered 100% procedural WebAudio SFX (including Dash whoosh, Witch-Time time-dilation shimmer, lightning crackle, boss war horn, 140 BPM boss synth, and persisted Mute toggle), UTC day/init state, run lifecycle, ghost replay, faction war percentages, live Siege Activity Feed, community mutator voting, and Reddit comment sharing.
- **Devvit Web + Redis Backend (`src/server/index.ts` → `dist/server/index.cjs`)**:
  - **UTC Daily Scheduler & Yesterday's Champion Coronation**: Automated daily post creation via `scheduler.tasks` + atomic Redis `SET NX` lock, automatically posting an official comment crowning yesterday's `#1 Champion` and winning Reddit Faction Order.
  - **Privacy & Retention**: 30-day (`30 * 86400`s) TTLs on all daily keys plus `onPostDelete` trigger cleanup.
  - **Run Session Anti-Cheat**: `POST /api/start-run` issues a `runId` + `startedAt`, validated on `POST /api/submit-run` (10s–180s duration bounds, score/kill clamping, per-user rate limit).
  - **Community & Social Hooks**:
    - **3-Faction Daily Subreddit War**: Every run contributes points to the player's chosen Order (`☀️ Solar Order`, `🔮 Arcane Conclave`, or `🗡️ Shadow Guild`) in Redis (`ef:factions:{sub}:{seed}`), rendered as live percentages on the Splash Card, Title, Victory screen, and Reddit Share Card.
    - **#1 Champion Ghost Replay**: Records 2 Hz `(x, y)` hero path (~1.4 KB) and renders the daily `#1` champion as a translucent golden phantom in-game and on the Minimap Radar.
    - **Subreddit Boss Siege & Live Activity Feed**: Collective daily Boss slain progress bar (`ef:bosses:{sub}:{seed}`) plus live ticker of recent Redditors striking/slaying today's Boss (`ef:siege-feed:{sub}:{seed}`).
    - **Community Vote — Tomorrow's Mutator**: Any signed-in Redditor can vote on tomorrow's `DailyModifier` at the end of a run (`POST /api/vote-mutator`) with live Redis tallies and a live UTC countdown teaser (`HH:MM:SS`).
    - **Daily Streak & 8-Bounty UTC Pool**: Per-Redditor Daily Streak (`🔥 X-Day Streak`) persisted in Redis and 3 deterministic Daily Bounties per UTC seed (including *2 Witch-Time Dodges*, *Reach Level 6*, and *Slay Today's Boss*).
    - **Enriched Reddit Build Share Card**: 1-tap `POST /api/share-comment` posts rank, streak, Hero Class, Faction Order, Daily Biome, score, bounties, live Faction War %, Siege bar, and Tomorrow's Vote Leader directly to the Reddit thread (zero external links).

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
- **Arch-Lich Phase II Enrage, Soul Obelisks & SUPERNOVA Ultimate**:
  - At `<50% HP`, the Daily Boss enters **Phase II Enrage** with double telegraphed slams, 12-way crimson spiral barrages, and **2 Soul Obelisks** shielding the Boss (`45%` damage reduction) until shattered (`+600 PTS` each).
  - Slaying enemies charges the **SUPERNOVA (`NOVA`)** ultimate ability for a screen-clearing triple shockwave, and triggers **Kill-Streak Announcer Milestones** (`25x RAMPAGE`, `50x UNSTOPPABLE`, `100x GODLIKE`).
- **3 Deterministic Daily Bounties, Evolution Codex & Post-Run DPS Breakdown**:
  - Every UTC daily seed rolls 3 deterministic mini-quests (*Detonate Barrels*, *Claim Shrines*, *Reach 30x Combo*, *Open Runic Chests*, *Slay Horde*) worth `+750 PTS` each and included in the Reddit Share Card (`🎯3/3`).
  - Pause Screen displays the **5-Recipe Evolution Synergy Codex**; Victory/Game Over screen displays the **Per-Weapon Damage Breakdown** (`SLASH`, `BOLTS`, `LIGHTNING`, `ORBIT`).
- **Tactical Minimap Radar, Off-Screen Pointers & Embedded TTF Typography**:
  - Real-time top-right radar tracking the Hero, Blood Shrines, Runic Barrels, `#1 Champion Ghost`, and Boss, plus edge-of-screen directional arrows for off-screen Shrines, Runic Chests, and Bosses.
  - Embedded `ChakraPetch-Bold.ttf` (SIL Open Font License 1.1) rendered via `load_ttf_font_from_bytes` with drop shadows.
- **GLSL100 Shaders (`crates/emberfall/src/shaders.rs`)**: `#version 100` hit-flash sprite material and `render_target` post-processing with **5-tap real-time Bloom/Glow**, **Fever Overdrive golden aura**, biome-tinted vignette, low-HP pulse, chromatic aberration, and shockwave distortion.
- **5 Weapon Evolutions (`crates/emberfall/src/weapons.rs`)**: Max-level weapon + passive synergies unlock *Meteor Barrage*, *Crimson Reaper*, *Inferno Vortex*, *Celestial Tempest* (`Chain Lightning Lv.3 + Soul Magnet Lv.3`), and *Soul Harvester* (`Vampiric Ward Lv.3 + Soul Magnet Lv.3`).
- **Runic Chests, Relics, Scorch Marks, Ambient Biome Weather & 140 BPM Boss Synth**:
  - Elites & Bosses drop illustrated `Runic Chests` (`runic_chest.png`) that grant passive Relics (*Vampiric Fang*, *Cryo Core*, *Crit Prism*, *Magnet Orb*) and trigger a global XP gem magnet pull.
  - Explosive barrels and Boss slams leave persistent scorch marks on the arena floor; each Daily Biome emits ambient weather particles, and the procedural WebAudio engine shifts into a **140 BPM dark-synth arpeggio** when the Arch-Lich awakens.
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

