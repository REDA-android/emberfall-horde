# Explorer 3 Handoff Report — Rust/Macroquad Juice, Audio, Shaders & CC0 Assets

**Summary**: `EMBERFALL: DAILY HORDE` already has a lean 931 KB WASM build (`opt-level = "z"`, `lto = true`, `strip = true`), a zero-dependency FNV-1a + xorshift64* daily PRNG (`rng.rs`), embedded PNGs via `include_bytes!()`, and floating crit damage numbers. The highest-ROI upgrades are: (1) **pure-Rust / GLSL100 post-processing & hit-flash/dissolve/shockwave shaders** via `macroquad::material::load_material` + `render_target`, (2) **hitstop freeze frames + trauma-squared shake + squash & stretch** in `main.rs`, (3) **fixing mobile Safari/Devvit WebView WebAudio unlock** + adding XP combo pitch-escalation and a procedural dark-fantasy drone/percussion loop (or `quad-snd` / `load_sound_from_bytes` with WAV fallback), and (4) **CC0 Kenney/OpenGameArt/itch.io SFX, dark-fantasy UI frames, and relic/rune icons**.

---

## 1. Observation

### A. Current Codebase Baseline (`emberfall-horde`)
- **`crates/emberfall/Cargo.toml`**: Uses `macroquad = "0.4"` (latest on crates.io is `0.4.16`), with `[profile.release]` already configured for size: `opt-level = "z"`, `lto = true`, `codegen-units = 1`, `panic = "abort"`, `strip = true`.
- **`dist/client/`**: `emberfall.wasm` is **931 KB**; `mq_js_bundle.js` is **37 KB**; `index.html` is **3.4 KB**.
- **`crates/emberfall/src/rng.rs`**: Implements a zero-dependency deterministic `Rng` (FNV-1a hash seeding + `xorshift64*`) with unit tests (`deterministic_365_days_and_ranks`) across 365 daily seeds.
- **`crates/emberfall/src/assets.rs`**: Uses `include_bytes!()` for 6 custom PNG sprites + 7 Kenney CC0 VFX PNGs (`fire_01`, `flame_01`, `spark_01`, `slash_01`, `magic_05`, `light_01`, `star_07`), avoiding external network fetches (CSP-safe).
- **`crates/emberfall/src/main.rs`**:
  - Already has: `hit_flash: f32` (lines 68, 504, 927), `TelegraphRing` (line 98), `SlashArc` (line 85), `LightningArc` (line 93), `Particle` (line 124), `combo` / `combo_timer` (lines 177–178), linear `shake` decay (line 707), XP gem magnetism (line 1118), and floating damage numbers with crit scaling (`scale: 26.0` vs `19.0`, lines 516–529, 1499).
  - Missing: `hitstop` (freeze frames), `load_material` / `gl_use_material` / `render_target` shaders, squash & stretch, XP combo pitch-escalation, and in-engine audio (`macroquad::audio`).
- **`dist/client/index.html` (lines 15–64)**: Audio is currently handled via a JS `miniquad_add_plugin` bridge (`emberfall_play_sfx(kind)`) using WebAudio `OscillatorNode`s (`kind 0..=8`). `audioCtx` is lazily created inside `tone()` without a dedicated `touchstart`/`pointerdown` user-gesture unlocker, and there is no background music or ambient drone.

### B. Pure-Rust / Macroquad / `wasm32-unknown-unknown` Crates & Techniques

#### 1. Deterministic PRNG for Daily Seeds
| Crate | Version (crates.io) | WASM `wasm32-unknown-unknown` Notes | Verdict for Emberfall |
|---|---|---|---|
| **Custom `Rng` (`src/rng.rs`)** | Zero deps | FNV-1a + `xorshift64*`, 0 KB overhead, no `getrandom` needed | **Keep** — already deterministic & tested |
| `oorandom` | `11.x` (PCG32, `#[no_std]`) | Minimalist PCG, no `getrandom` dependency | Good alternative if PCG statistical quality is ever needed |
| `fastrand` | `2.4.1` | Simple WyRand; manual seeding via `fastrand::Rng::with_seed(u64)` avoids JS entropy | Lightweight, but unnecessary over existing `rng.rs` |
| `rand` + `rand_chacha` | `rand 0.9.5` / `0.10`, `rand_chacha 0.10` | `wasm32-unknown-unknown` fails unless `getrandom` has `wasm_js` feature OR `os_rng`/`sys_rng` default feature is disabled (`SeedableRng` only). Adds binary size | **Avoid** — unnecessary bloat for daily arcade seeds |

#### 2. Audio in Macroquad WASM & Mobile Safari / Devvit WebView Quirks
- **`macroquad::audio` (`quad-snd`) quirks**:
  - **Ogg Vorbis fails on Safari / iOS WebKit**: Macroquad's book (`mq.agical.se/ch12-audio.html`) explicitly notes Ogg Vorbis is *not* supported by Safari; Safari requires **WAV** (or MP3 if decoded via browser). For `include_bytes!()` + `load_sound_from_bytes`, **16-bit mono 22050 Hz WAV** (or ADPCM/procedural PCM) works universally across Chrome, Firefox, Safari, iOS WebView, and Android WebView.
  - **Autoplay / `AudioContext` suspension on iOS Safari & Reddit WebView**: Browsers block `AudioContext` until a synchronous user gesture (`pointerdown` / `touchend` / `keydown`). Calling `audioCtx.resume()` inside `requestAnimationFrame` (where Macroquad's game loop runs) is often ignored on iOS Safari unless `AudioContext` was created/unlocked inside a DOM input event listener.
  - **Macroquad `load_sound` async queueing bug (Issue #440)**: Loading many audio files via `load_sound()` can stall or queue playback; disabling `macroquad` audio feature (`default-features = false`) historically didn't strip `quad-snd` cleanly.
- **Pure-Rust Procedural Audio Crates**:
  - `sfxr` (`0.1.3`, `bzar/sfxr-rs`): Reimplementation of DrPetter's SFXR. Generates `[f32; N]` buffers in pure Rust; caveat from author: on `wasm32-unknown-unknown` older builds needed `sin`/`sinf`/`pow` imports unless using `libm`. Better yet: a **~60-line pure-Rust WAV synth** (sine/square/triangle/noise + ADSR + pitch sweep + 44-byte RIFF/WAV header) can either feed `macroquad::audio::load_sound_from_bytes` or keep the existing zero-KB JS WebAudio bridge upgraded with:
    1. Dedicated `window.addEventListener('pointerdown', unlockAudio, { once: true })` + `touchstart` + `keydown` playing a 1-sample silent buffer to permanently unlock iOS/Android WebView `AudioContext`.
    2. Pitch parameter passed from Rust (`emberfall_play_sfx_pitched(kind, pitch_mul)`) for **XP gem combo pitch-escalation** (e.g., pentatonic/chromatic climb `1.0 + combo * 0.04`).
    3. Procedural dark-fantasy WebAudio drone + low-pass filtered ember crackle / war-drum pulse (0 KB WASM/asset cost!).
  - `tinyaudio` (`0.1.2`) / `fundsp` (`0.23.0`): `tinyaudio` requires `wasm-bindgen` (incompatible with Macroquad's `miniquad` JS bundle without custom glue). `fundsp` is `no_std` DSP, great for offline/procedural buffer generation if needed, though overkill for simple SFX.

#### 3. Screen-Space & Sprite GLSL100 Shaders in Macroquad (`load_material`, `render_target`)
- **Critical WebGL1 / Mobile Constraint**: Macroquad / Miniquad compiles shaders as **GLSL 100 (`#version 100`)** for WebGL1 compatibility (`good-web-game` / `macroquad/examples/custom_material.rs` / `post_processing.rs`).
  - Always include `#version 100` and `precision mediump float;` (mobile GPUs fail to compile GLSL100 fragment shaders without `precision` qualifiers).
  - Macroquad auto-injects uniforms: `_Time`, `Model`, `Projection`, `Texture`, `_ScreenTexture`.
- **Render Target Y-Flip Caveat (`not-fl3/macroquad#171`)**:
  - Drawing a `Camera2D { render_target: Some(rt), .. }` and then drawing `rt.texture` to the default camera can appear upside-down unless `dest_size` / `flip_y: true` (or negative zoom `y`) is handled consistently. Test on both native & WebGL!
- **High-ROI Shaders for Emberfall (GLSL100)**:
  1. **Post-Processing Pass (`render_target` -> screen)**:
     - **Dark Fantasy Vignette + Ember Glow**: Darkens edges (`smoothstep` radial falloff) + subtle warm orange/crimson pulse when HP < 30% or during `BloodMoon` / Boss phase.
     - **Chromatic Aberration on Heavy Hits**: Offset R/B channels radially by `u_aberration * (uv - 0.5)` decaying over 0.15s on player damage or boss slam.
     - **Shockwave / Ripple**: Expand a ring `dist = length(uv - u_epicenter)` distorting `uv` along `normalize(uv - u_epicenter)` on Boss spawn, level-up nova, or Boss death.
  2. **Per-Sprite Material (`gl_use_material`)**:
     - **True White/Ember Hit Flash**: Currently `draw_texture_ex` tinting only multiplies colors (darkening already dark sprites). A 10-line GLSL100 fragment shader (`mix(tex.rgb, vec3(1.0, 0.92, 0.8), u_flash)`) produces crisp Vampire-Survivors-style white hit flashes!
     - **Ash / Ember Dissolve Death Shader**: Discard pixels where `hash(uv) < u_dissolve` and color the threshold edge `u_dissolve..u_dissolve+0.08` bright ember orange (`vec3(1.0, 0.4, 0.05)`).

#### 4. Game Feel / "Juice" Patterns in Rust/Macroquad
- **Hitstop (Freeze Frames)**: Add `hitstop: f32` to `GameState`. When `hitstop > 0.0`, decrement `hitstop -= raw_dt`, still decay `shake` and render, but skip simulation `update(dt)` (or scale `dt` by `0.05` for micro-slowmo). Trigger `0.035s` on crit kills, `0.08s` on Boss phase/death, `0.05s` on player hit.
- **Trauma-Based Screen Shake**: Replace linear `shake` with `trauma: f32` (`0.0..=1.0`), decaying linearly (`trauma = (trauma - dt * 1.8).max(0.0)`), and apply `shake_offset = max_px * trauma * trauma * sin/noise` (quadratic/cubic curve feels much punchier than linear).
- **Squash & Stretch**: Scale sprite `dest_size` by `(1.0 + squash, 1.0 - squash)` decaying back to `(1.0, 1.0)` on dash, enemy spawn pop-in, or hit impact.
- **Boss / Elite Telegraphing**: Already has `TelegraphRing`; enhance with an inner filling progress circle (`fill_ratio = 1.0 - timer / max_timer`) so mobile players can read exact dodge timing at a glance.

#### 5. WASM Size Optimization & Mobile WebView
- `Cargo.toml` already uses the optimal release profile (`opt-level = "z"`, `lto = true`, `codegen-units = 1`, `panic = "abort"`, `strip = true`).
- **`wasm-opt` (Binaryen) with Rust 1.82+ / 1.87+**: Modern `rustc` emits `bulk-memory` instructions by default. Running `wasm-opt -Oz` without flags fails with `"Bulk memory operations require bulk memory"`. **Fix**: pass `--all-features` (or `--enable-bulk-memory --enable-nontrapping-float-to-int`) to `wasm-opt -Oz`. Typically saves ~70–150 KB on a ~930 KB Macroquad WASM.
- **Texture Atlasing & PNG Quantization**:
  - Call `build_textures_atlas()` after loading textures in `GameAssets::load()` (if using `draw_texture_ex` without custom per-texture shaders) to reduce WebGL draw calls on mobile.
  - Quantize baked PNGs (`pngquant` / `oxipng`) and resize sprites to max display size (e.g., 128x128 or 256x256 instead of 1024x1024) — baked PNGs dominate the 931 KB WASM binary.

---

### C. CC0 Dark Fantasy / Survivor-Like Assets & Open-Source Repos

#### 1. CC0 Sound Effects (No Attribution Required, Safe for Devvit)
- **Kenney Sound Packs (CC0 1.0)**:
  - *Impact Sounds* (`kenney.nl` / Godot AssetLib `1841`): Heavy thuds, metal clangs, flesh/stone hits.
  - *RPG Audio* (`kenney.nl` / `gamesounds.xyz`): Sword swings, shield blocks, footsteps, potion/coin pickups.
  - *Interface Sounds* (`github.com/Calinou/kenney-interface-sounds`, 100 files): Crisp clicks, hovers, level-up confirmations.
- **OpenGameArt CC0 SFX**:
  - *RPG Sound Pack* by `artisticdude` (CC0 1.0, ~12.5 MB OGG) — swords, spells, UI, monster hits.
  - *100 CC0 SFX* (`opengameart.org/content/100-cc0-sfx`).
  - *8-bit Sound Effect Pack* by `OwlishMedia` (CC0) & *Battle Sound Effects* by `Ogrebane` (CC0 option).
  - *War on Water: SNDFX* by `yd` & *Tiny Naval Battle Sounds Set* by `qubodup` (CC0 explosions/impacts).
- **Generators (CC0 output)**:
  - **ChipTone** (`sfbgames.itch.io/chiptone`) & **jsfxr** (`sfxr.me`) — export tiny 22 kHz mono WAVs for crisp retro-dark-fantasy hits/pickups.

#### 2. CC0 Dark Fantasy Music Loops
- **OpenGameArt CC0**:
  - *Boss Battle 10 [Retro]* by `nene` (`opengameart.org/content/boss-battle-10-retro`, CC0 1.0).
  - *Creepy Forest (F)* by `Brandon Morris` (`opengameart.org/content/creepy-forest-f`, CC0).
  - *Besai Crystal Gardens 2 - Forbidden Pathways* by `Tozan` (CC0).
  - *8-bit Music for GameDev - Perilous Dungeon* (CC0 1.0, Wikimedia Commons / OGA).
  - *MOD/XM Chiptune Modules* (`modarchive.org` filtered by `query=cc0`) — tiny (20–80 KB!), though Macroquad needs a tracker decoder or pre-render to compressed audio.
- **Note**: `RandomMind` on OpenGameArt has great medieval/battle loops (verify CC0 vs CC-BY per track before bundling).

#### 3. CC0 UI Icons, Relic/Weapon Runes & Dark Fantasy Frames
- **Itch.io CC0 Dark Fantasy UI & Icons**:
  - *Tiny RPG - Dark Dwellers GUI* by `tiopalada` (`tiopalada.itch.io/tiny-rpg-dark-dwellers-gui`, **CC0 1.0**, No AI): 9-slice frames, HP/Mana/XP bars, equipment/relic frames, buttons.
  - *Fantasy Pixel UI - 500 RPG Pixel Art UI Elements* by `Riclmbr` (`riclmbr.itch.io/fantasy-pixel-ui-500-rpg-pixel-art-ui-elements`, **CC0**): Stone & gold buttons, 9-slice panels, 120 icons.
  - *Fantasy RPG & Dungeon Starter Pack* by `ArtFromPixels` (`artfrompixels.itch.io/fantasy-rpg-dungeon-mega-pack-free`, **CC0**, 32x32 & 48x48, No AI).
  - *Game Icon Pack (800+ Rounded Free Icons)* (CC0 on itch.io).
- **Game-icons.net (4,180+ SVG/PNG RPG/Relic/Weapon icons)**:
  - **License Caveat**: Mostly **CC-BY 3.0** (requires a one-line credit like `"Icons by Lorc, Delapouite, sbed via game-icons.net"` in an About/Credits modal), with some Public Domain. Extremely clean for Emberfall weapon/upgrade cards!

#### 4. Open-Source Macroquad Survivor-Like / Roguelite Repos (Architecture & Shader Reference)
- **`schweller/vs-demake-lowrezjam2023`** (GitHub): Vampire-Survivors-like built in pure Rust + Macroquad for WASM32; clean example of 4-minute run loop, XP/level-up passive upgrades, and stopwatch timing on WASM.
- **`nearbycoder/Cinderwake`** (GitHub): Action roguelite on Macroquad with fixed-step simulation, rendering composition, and boss combat.
- **`christianWissmann85/octoplat`** (GitHub): Procedurally-generated roguelite built with Rust + Macroquad.
- **`alexmozaidze/macroquad-canvas`** (GitLab): Resolution-independent fixed-aspect canvas library for Macroquad `0.4` using `render_target` (great reference for handling mobile portrait/landscape scaling cleanly).
- **`not-fl3/macroquad/examples/`**: `post_processing.rs` (`render_target` + screen shader) and `custom_material.rs` (`#version 100` GLSL + `UniformDesc`).

---

## 2. Logic Chain
1. **CSP & Bundle Size Constraint**: Devvit Web enforces strict CSP (no external CDN fetches) and mobile Reddit users abandon slow-loading WebViews. `emberfall.wasm` is already 931 KB via `include_bytes!()`. Therefore, **procedural GLSL100 shaders + procedural WebAudio (or tiny 22kHz WAVs) + `wasm-opt -Oz --all-features`** deliver maximum visual/audio impact with near-zero byte cost.
2. **Mobile Safari / Devvit WebView Audio**: Because `index.html` currently creates `AudioContext` inside `tone()` (invoked from Macroquad's `requestAnimationFrame` loop), iOS WebKit can leave `audioCtx.state === 'suspended'`. Adding a synchronous `pointerdown`/`touchstart` unlocker + passing `combo` pitch to `emberfall_play_sfx` immediately fixes iOS audio and adds Vampire-Survivors-style XP gem doppler/combo escalation.
3. **Visual Impact Without Asset Bloat**: `main.rs` already tracks `hit_flash` and `shake`, but lacks GPU white-flash (`load_material`), `hitstop`, quadratic `trauma` shake, and a post-process vignette/chromatic-aberration pass (`render_target`). Adding these ~150 lines of Rust + GLSL100 transforms combat feel at <5 KB WASM cost.

---

## 3. Caveats
- `read_url_content` had no external DNS in this sandbox environment, so remote file contents (e.g., exact lines of `macroquad/examples/post_processing.rs`) were verified via Google search snippets rather than raw file downloads.
- `wasm-opt` was not executed against `dist/client/emberfall.wasm` in this read-only survey.
- Mobile Safari / Reddit iOS WebView audio behavior was not tested on physical hardware; recommendations are based on WebKit autoplay policy and `tinyaudio`/`quad-snd` documentation.
- `Game-icons.net` is **CC-BY 3.0** (not CC0) — requires an in-game credit line if used, whereas Kenney / `tiopalada` / `Riclmbr` / `artisticdude` are **CC0**.

---

## 4. Conclusion (Prioritized Recommendations)
1. **P0 — Mobile WebAudio Unlock + XP Pitch Escalation + Dark Drone (`dist/client/index.html` + `main.rs`)**: Add synchronous `pointerdown`/`touchstart`/`keydown` `AudioContext` unlock; pass `combo` / pitch multiplier on XP gem pickup (`kind === 8`); add a subtle toggleable WebAudio dark-fantasy bass drone + war-drum pulse.
2. **P0 — Hitstop + Trauma Shake + Squash & Stretch (`crates/emberfall/src/main.rs`)**: Add `hitstop: f32` (35–80 ms freeze on crits/boss/player hit), `trauma * trauma` camera shake, and sprite squash & stretch on dash/spawn/hit.
3. **P1 — GLSL100 Shaders (`crates/emberfall/src/shaders.rs` + `main.rs`)**:
   - **Hit-flash material** (`mix(tex.rgb, vec3(1.0, 0.92, 0.8), u_flash)`) for crisp white enemy flashes.
   - **Post-processing `render_target` shader** with `#version 100` + `precision mediump float;`: dark vignette + low-HP crimson pulse + chromatic aberration on heavy hits + expanding shockwave on Boss spawn/level-up.
4. **P1 — WASM Size Hygiene**: Run `wasm-opt -Oz --all-features dist/client/emberfall.wasm -o dist/client/emberfall.wasm` in the build pipeline and optimize baked PNG dimensions/compression (`oxipng`/`pngquant`).
5. **P2 — CC0 Dark Fantasy UI & Audio Polish**: Adopt `tiopalada`'s *Tiny RPG - Dark Dwellers GUI* (CC0) or `Riclmbr`'s *Fantasy Pixel UI* (CC0) for level-up upgrade cards/frames, and *Kenney Impact/RPG Audio* (CC0, 22 kHz mono WAV) for heavy boss/sword impacts if upgrading beyond WebAudio synths.

---

## 5. Verification Method
- **PRNG & Unit Tests**: `cargo test` in `/home/antigravity/REDA-android/emberfall-horde/crates/emberfall`.
- **WASM Build & Size Check**:
  ```bash
  cargo build --manifest-path crates/emberfall/Cargo.toml --target wasm32-unknown-unknown --release
  wasm-opt -Oz --all-features target/wasm32-unknown-unknown/release/emberfall.wasm -o dist/client/emberfall.wasm
  ls -lh dist/client/emberfall.wasm
  ```
- **Shader & Mobile WebView Validation**: Serve `dist/client/` locally and test in Chrome DevTools mobile emulation + iOS Safari / WebKit to confirm (1) GLSL100 shaders compile with zero WebGL warnings, (2) `render_target` is right-side up, and (3) audio plays on first tap.
