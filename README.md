# EMBERFALL: DAILY HORDE

**EMBERFALL: DAILY HORDE** is a 90-second Dark Fantasy Action Horde Survivor ARPG written in **100% pure Rust** using [`not-fl3/macroquad`](https://github.com/not-fl3/macroquad) compiled to **WebAssembly (`.wasm`)**, built for **Reddit's Game of the Day Hackathon 2026** on **Devvit Web** and **Redis**.

---

## Architecture (Devvit Web + Redis + Pure Rust)

- **100% Pure Rust WASM Client (`crates/emberfall/` → `dist/client/emberfall.wasm`)**: All game loop logic, 3 playable Hero Classes, 3 Daily Biomes with bespoke seamless tiled floors (*Obsidian Sanctuary*, *Volcanic Rift* magma basalt, *Frost Abyss* runic ice), 3 distinct Daily Boss sprites (*Malakor*, *Ignis Rex*, *Skadi*), 3 animated Faction Companions, 8 illustrated gold-framed skill icons (`icons_atlas.png`), Runic Explosive Barrels, AI hordes, GLSL100 shaders (including toggleable CRT Arcade scanline/phosphor filter), particle explosions, audio/voice-blip triggers, tactical Minimap Radar, Talking Portrait Banter Boxes, and sculpted Hero Avatar Medallion HUD are compiled from Rust with zero HTML/JS/CSS game logic and all PNGs + Open-Source TTF typography embedded via `include_bytes!()`.
- **Devvit Web Entrypoints (`dist/client/`)**:
  - `splash.html` + `splash.js` + `splash_hero.webp` (`default` inline feed card, zero WASM, zero inline scripts): Features a dark-fantasy key-art hero banner, dynamically adapts its color palette to today's UTC Daily Biome (*Obsidian* amber/gold, *Volcanic* lava/ember, *Frost* ice/cyan), shows today's Boss, Mutator, Community Faction Blessing, 3 Daily Bounties, live 3-segment Faction War percentage bar, Subreddit Boss Siege progress & live Siege Activity Feed, Top 3 Champions, Tomorrow's Dungeon Countdown & Vote Leader teaser, plus an interactive `? Guide` briefing drawer before launching fullscreen via `requestExpandedMode(event, 'game')`.
  - `index.html` + `game_bridge.js` (`game` expanded fullscreen host, zero inline scripts): Loads `mq_js_bundle.js` + `emberfall.wasm` and registers the `emberfall_bridge` plugin for the **4-track procedural WebAudio Dark-Synthwave / Modern Chiptune engine** (Bass + 16-bit Arp + Heroic Lead + Percussion through a `DynamicsCompressorNode` + procedural Cathedral Reverb `ConvolverNode`, 3 distinct Biome musical themes, 2-phase Boss battle themes 140/152 BPM, Victory/Defeat fanfares, 16-bit Talking Portrait voice blips, and persisted Mute + CRT preferences), UTC day/init state, run lifecycle, ghost replay, faction war percentages, live Siege Activity Feed, community mutator voting, and Reddit comment sharing.
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

- **3 Playable Hero Classes & Animated Faction Companions (`crates/emberfall/src/weapons.rs`)**:
  - **☀️ Sun Knight (*Solar Order*) + Solar Phoenix Hatchling**: `130 HP`, balanced melee + fireball starter (`Arcane Slash Lv.1` + `Ember Bolts Lv.1`).
  - **🔮 Pyromancer (*Arcane Conclave*) + Astral Owl**: `95 HP`, long-range spellcaster (`Ember Bolts Lv.1` + `Chain Lightning Lv.1` + `Soul Magnet Lv.1`).
  - **🗡️ Shadow Assassin (*Shadow Guild*) + Shadow Dire Wolf Pup**: `105 HP`, high-mobility rogue (`Arcane Slash Lv.1` + `Runic Orbit Lv.1` + `Inferno Dash Lv.2` + `Arcane Haste Lv.1`).
  - Each companion spring-follows the hero and fires themed auto-homing bolts at nearby enemies!
- **3 Bespoke Daily Biomes, 3 Unique Boss Sprites & Runic Explosive Barrels (`crates/emberfall/src/rng.rs`)**:
  - **Obsidian Sanctuary (`MALAKOR, VOID LICH`)**: Arcane Barrels (`+25%` shockwave damage) + Void Storm Lightning.
  - **Volcanic Rift (`IGNIS REX, MAGMA TITAN`)**: Magma Barrels (`+30%` explosion radius) + Magma Geyser Eruptions + cracked magma basalt floor.
  - **Frost Abyss (`SKADI, GLACIAL EMPRESS`)**: Cryo Barrels (chills & slows hit enemies by `42%` for `3.5s`) + Blizzard Squalls + glowing runic ice floor.
  - Barrels chain-detonate nearby barrels when destroyed by attacks or Dash!
- **Sculpted Hero Avatar Medallion HUD, Talking Portrait Banter Box & Illustrated Upgrade Cards**:
  - Top-left sculpted HUD displays an animated Hero portrait medallion (breathing, crimson damage pulse, golden Fever halo), beveled specular HP/Fever bars, and equipped 18×18 skill icons from `icons_atlas.png`.
  - Non-blocking 2.8s **Talking Portrait Banter Boxes** with 16-bit retro voice blips trigger on run start, SUPERNOVA casts, Boss arrival, and Boss Phase II Enrage.
  - Level-Up cards display 64×64 gold-framed skill illustrations, star rank meters (`★`), and a glowing `★ EVOLVES -> ...!` badge whenever a choice unlocks a Weapon Evolution.
  - **Liquid Vial SUPERNOVA Orb (`NOVA`)** visibly fills with animated waving ember liquid as you slay the horde.
- **Boss Phase II Enrage, Soul Obelisks & Kill-Streak Milestones**:
  - At `<50% HP`, the Daily Boss enters **Phase II Enrage** (shifting the music to 152 BPM) with double telegraphed slams, 12-way crimson spiral barrages, and **2 Soul Obelisks** shielding the Boss (`45%` damage reduction) until shattered (`+600 PTS` each).
  - Slaying enemies charges the **SUPERNOVA (`NOVA`)** ultimate ability for a screen-clearing triple shockwave, and triggers **Kill-Streak Announcer Milestones** (`25x RAMPAGE`, `50x UNSTOPPABLE`, `100x GODLIKE`).
- **3 Deterministic Daily Bounties, Evolution Codex & Post-Run DPS Breakdown**:
  - Every UTC daily seed rolls 3 deterministic mini-quests worth `+750 PTS` each and included in the Reddit Share Card (`🎯3/3`).
  - Pause Screen displays the **5-Recipe Evolution Synergy Codex** and Today's Dungeon Briefing; Victory/Game Over screen displays the **Per-Weapon Damage Breakdown** (`SLASH`, `BOLTS`, `LIGHTNING`, `ORBIT`).
- **Tactical Minimap Radar, Off-Screen Pointers & Embedded TTF Typography**:
  - Real-time top-right radar tracking the Hero, Blood Shrines, Runic Barrels, `#1 Champion Ghost`, and Boss, plus edge-of-screen directional arrows for off-screen Shrines, Runic Chests, and Bosses.
  - Embedded `ChakraPetch-Bold.ttf` (SIL Open Font License 1.1) rendered via `load_ttf_font_from_bytes` with drop shadows.
- **GLSL100 Shaders (`crates/emberfall/src/shaders.rs`)**: `#version 100` hit-flash sprite material and `render_target` post-processing with **5-tap real-time Bloom/Glow**, **Fever Overdrive golden aura**, optional **CRT Arcade scanline & phosphor shader (`C` / `CRT` button)**, biome-tinted vignette, low-HP pulse, chromatic aberration, and shockwave distortion.
- **5 Weapon Evolutions (`crates/emberfall/src/weapons.rs`)**: Max-level weapon + passive synergies unlock *Meteor Barrage*, *Crimson Reaper*, *Inferno Vortex*, *Celestial Tempest* (`Chain Lightning Lv.3 + Soul Magnet Lv.3`), and *Soul Harvester* (`Vampiric Ward Lv.3 + Soul Magnet Lv.3`).
- **Runic Chests, Relics, Scorch Marks, Ambient Biome Weather & 4-Track Procedural Dark-Synthwave**:
  - Elites & Bosses drop illustrated `Runic Chests` (`runic_chest.png`) that grant passive Relics (*Vampiric Fang*, *Cryo Core*, *Crit Prism*, *Magnet Orb*) and trigger a global XP gem magnet pull.
  - Explosive barrels and Boss slams leave persistent scorch marks on the arena floor; each Daily Biome emits ambient weather particles and plays its own bespoke 4-track procedural synth soundtrack.
- **Blood Shrines & Kill-Streak Fever Meter**: 1–2 deterministic Blood Shrines per daily seed (2s channel spawns an elite for `+0.5x` score multiplier) + Fever Meter (`x1.0` to `x2.5` multiplier).

---

## Controls

| Action | Mobile / Touch | Desktop |
| --- | --- | --- |
| **Select Hero Class** | Tap Class Card on Title Screen | Click Card or press `1` / `2` / `3` |
| **Move Hero** | Drag anywhere (1-thumb virtual joystick) | `WASD` or Arrow Keys |
| **Auto-Attack** | Automatic (targets nearest enemies & barrels) | Automatic |
| **Inferno Dash** | Tap **DASH** button (bottom-right) | `Space` or Right-Click |
| **Supernova Ultimate** | Tap **NOVA** liquid vial (left of DASH when charged) | `Q` or `E` |
| **Pick Level-Up Card** | Tap card | Click card or press `1` / `2` / `3` |
| **Pause / Info / Mute / CRT** | Tap top-right `||` / `?` / `S` / `CRT` HUD buttons | `P` / `Esc` (Pause), `M` (Mute), `C` (CRT Filter) |

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

