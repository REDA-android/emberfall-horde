# HENRY'S ARCADE STREETS — EMBERFALL EDITION (`v0.6.0`)

**HENRY'S ARCADE STREETS (Emberfall Edition)** is a fast-paced **16-bit Retro 2D Side-Scroller Beat'em Up** built for **Reddit Devvit Web + Redis** and playable instantly at 60 FPS on Desktop, Mobile Reddit, and Gamepad — with zero external asset requests and strict Devvit CSP compliance.

- **Play Instant Web Preview (Vercel):** https://emberfall-horde.vercel.app
- **Reddit Test Subreddit:** https://www.reddit.com/r/emberfall_horde_dev
- **Repository:** https://github.com/REDA-android/emberfall-horde

---

## Highlights & Arcade Features

- **Authentic 16-Bit Harbor Quay 2D Side-Scroller (`dist/client/arcade_engine.js`)**:
  - **4-Layer Parallax Boulevard:** Rolling royal-blue harbor waves with mooring bollards, stone quay sidewalk, colorful European harbor rowhouses (*Barbershop* with spinning pole, yellow/blue/red townhouses with flower boxes & chimney smoke, shatterable *FANCY* storefront), flapping pixel seagulls, distant castle fortress, and dynamic **3-Act Time-of-Day Progression** (*Act I: Harbor Docks* → *Act II: Sunset Boulevard* → *Act III: Syndicate Night*).
  - **2.8s Instant Action Intro Cinematic + Live 1991 Attract Mode:** Syndicate limousine drops suited goons (`"SEIZE THE HARBOR!"`), Henry sky-drops onto the quay (`"READY FOR THE RELOAD?"`), and combat begins with zero loading delay.
  - **Iconic Full-Screen `"NEW POWER UP: ARM RELOAD"`:** Charges via the tall left-edge **Vintage Brass Mercury Thermometer**, triggering the full-screen golden thumbs-up power-up banner, mechanical gear-click SFX, and 8 seconds of invincible blazing Super Punches!
- **Deep Beat'em Up Combat & Street Comedy**:
  - **4 Playable Fighter Styles:** **Golden Titan** (`[K]` Power Kick), **Royal Ranger** (`[K]` Cyan Blitz Dash with afterimages), **Crimson Fist** (`[K]` Rising Dragon Uppercut), and unlockable **Secret Agent** (unlocked at `5,000 PTS` or by defeating Don Mustachio).
  - **Flying Mustache Boomerangs & Golden Mustache Shield:** Punch incoming mustaches mid-air (`PARRY! +250`) to deflect them right back into suited goons, or collect Golden Mustaches for a 5.5s orbiting auto-parry shield!
  - **Grab & Bowl-Throw + Destructible Street Props:** Grab stunned goons (`[O]`) and bowl them through enemy lines (`STRIKE!`), knock enemies over the quay wall into the harbor (`PLOUF!`), jump onto the yellow **Telephone Booth** & parked **White Car** roofs, smash them for **Pizza (+1 Heart)** / **Steel Pipes**, or launch **Trash Cans** across the street!
  - **2-Phase Harbor Kingpin Boss (`DON MUSTACHIO`):** Arrives by limousine at 25 KOs with a Phase II **Harbor Rage** mode, telegraphed **Seismic Slam** ring to jump over, and a **Counter-Stun (`+600 PTS`)** punish window!
- **Procedural 16-Bit Chiptune Audio Engine & Mixer**:
  - Zero external `.mp3`/`.wav` files — synthesized WebAudio bassline, lead arpeggio, percussion, punch/parry/splash/glass SFX, mechanical *Arm Reload* clicks, and live **Music / SFX Volume Sliders** in the Pause menu.
- **Reddit Devvit Web + Redis Backend (`src/server/index.ts`)**:
  - Strict Devvit CSP compliance (`0` inline `<script>`/`<style>`/`style=`, `0` external URLs, `requestExpandedMode` splash card).
  - Daily UTC Mutators (`MUSTACHE STORM`, `TURBO RELOAD`, `HEAVY SYNDICATE`, `CAR SMASH BONUS`), Daily Top Champions Leaderboard, Arcade Medals, and 1-tap Reddit Chronicle comment sharing.

---

## Controls (Keyboard · Mobile Touch · Gamepad)

| Action | Keyboard | Mobile Touch | Gamepad |
| --- | --- | --- | --- |
| **Move / Lane Shift** | `WASD` or Arrow Keys | Bottom-Left `◀` / `▶` Pad | Left Stick / D-Pad |
| **Jump (Roof / Dodge Slam)** | `Space` | `▲` Button | `A` |
| **Punch Combo / Parry / Grab-Throw** | `O` or `J` | Red `O` Button | `X` or `B` |
| **Signature Special / Throw Pipe** | `K` | Blue `K` Button | `Y` |
| **Trigger ARM RELOAD (100% Thermo)** | `L` or `Q` | Gold `L` Button | `RB` or `LB` |
| **Pause / Audio Mixer / CRT Toggle** | `P` or `Esc` | Top-Center `PAUSE` | `Start` |

---

## Build, Verify & Deploy

```bash
# 1. Install dependencies
npm install

# 2. Run Rust unit tests & TypeScript typecheck
cargo test --manifest-path crates/emberfall/Cargo.toml
npm run typecheck

# 3. Build server & assets
npm run build

# 4. Publish & install on Reddit Devvit
npx devvit login
npx devvit publish --public --version 0.6.0
npx devvit install emberfall_horde_dev emberfall-horde@0.6.0
```
