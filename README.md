# EMBERFALL: DAILY HORDE

**EMBERFALL: DAILY HORDE** is a 90-second Dark Fantasy Action Horde Survivor ARPG written in **100% pure Rust** using the open-source [`not-fl3/macroquad`](https://github.com/not-fl3/macroquad) engine compiled to **WebAssembly (`.wasm`)**, built for **Reddit's Game of the Day Hackathon 2026** on **Devvit Web** and **Redis**.

---

## Features

- **100% Pure Rust WebGL / WebAssembly Engine:** Game loop, AI hordes, 5 escalating weapons, particle explosions, floating critical damage numbers, screen-shake, hit-flash, and in-game UI are all compiled from Rust (`crates/emberfall/`) to `dist/client/emberfall.wasm`.
- **High-Detail Illustrated Sprites + CC0 Kenney VFX:** Custom Dark Fantasy Arcane Knight hero, Lava Imps, Obsidian Golems, Arcane Shadow Wraiths, a massive **Ember Lich Dragon Overlord Daily Boss**, seamless basalt dungeon floor tiling, and CC0 particle textures from [`Calinou/kenney-particle-pack`](https://github.com/Calinou/kenney-particle-pack).
- **90-Second Daily Dungeon (UTC Seed):** Every UTC day generates a shared **Daily Modifier** (*Blood Moon*, *Arcane Surge*, *Swarm Rush*, *Titan's Wrath*), deterministic enemy waves, upgrade card pool, and **Daily Boss awakening at 55s**.
- **1-Thumb Mobile Controls:** Drag anywhere on screen to dodge hordes (virtual thumbstick) while your hero auto-aims & unleashes Arcane Slash, Ember Bolts, Chain Lightning, and Runic Orbits. Tap **DASH** (or press `Space`) for an invincible Inferno Dash shockwave!
- **Devvit Web + Redis Community Integration:** Subreddit Daily Leaderboard, daily streaks, subreddit Boss-slain counter, Rank `E → SSS`, and 1-tap **Post Score to Reddit Comments** (zero external links, strict CSP compliant).

---

## Controls

| Action | Mobile / Touch | Desktop |
| --- | --- | --- |
| **Move Hero** | Drag anywhere (1-thumb virtual joystick) | `WASD` or Arrow Keys |
| **Auto-Attack** | Automatic (targets nearest enemies) | Automatic |
| **Inferno Dash** | Tap **DASH** button (bottom-right) | `Space` or Right-Click |
| **Pick Level-Up Card** | Tap card | Click card or press `1` / `2` / `3` |

---

## Build & Deployment (Devvit Web)

```bash
# 1. Install Devvit Web server dependencies
npm install

# 2. Run Rust unit tests (verifies 365 consecutive UTC days + Rank E->SSS)
cd crates/emberfall && cargo test && cd ../..

# 3. Build Rust -> WebAssembly (dist/client/emberfall.wasm) & Devvit server (dist/server/index.cjs)
npm run build

# 4. Playtest & Upload to Reddit Developers
npx devvit login
npx devvit playtest <your-public-test-subreddit>
npx devvit upload
```

---

## Open-Source Credits & Licenses

- **Game Engine:** [`not-fl3/macroquad`](https://github.com/not-fl3/macroquad) — Dual-licensed under MIT / Apache-2.0.
- **VFX Particle Textures:** [`Calinou/kenney-particle-pack`](https://github.com/Calinou/kenney-particle-pack) (by Kenney) — Licensed under **CC0 1.0 Universal (Public Domain)**.
- **Platform SDK:** [`@devvit/web`](https://developers.reddit.com/) — Reddit Developer Platform.
