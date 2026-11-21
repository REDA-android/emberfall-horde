# Challenger 2 Handoff — M2 Empirical Verification

**Verdict**: `REQUEST_CHANGES`

## 1. Observation
- **Native test suite**: Ran `cargo test --manifest-path crates/emberfall/Cargo.toml`:
  - Exit code `0`, `7 passed; 0 failed; 0 ignored`.
  - 1 compiler warning: `warning: method accent is never used` at `crates/emberfall/src/weapons.rs:92:12` (`EvolutionId::accent`).
- **WASM32 target check**: Ran `cargo check --manifest-path crates/emberfall/Cargo.toml --target wasm32-unknown-unknown`:
  - Exit code `101`: `error[E0463]: can't find crate for core` / `the wasm32-unknown-unknown target may not be installed`.
- **`#[cfg(target_arch = "wasm32")]` vs `#[cfg(not(target_arch = "wasm32"))]` FFI**:
  - Inspected `crates/emberfall/src/main.rs:12-126` and `dist/client/index.html:244-455`: all 16 `extern "C"` imports have matching `#[cfg(not(target_arch = "wasm32"))]` stubs and matching JS `env.emberfall_*` bindings.
- **Ghost replay recording & playback**:
  - `crates/emberfall/src/main.rs:1008-1012`: `ghost_record` appends 2 `i16`s every `0.5s` only while `self.ghost_record.len() < 400` (`<= 200` points). `main.rs:1585` computes `ghost_count = (self.ghost_record.len() / 2) as u32`.
  - `crates/emberfall/src/main.rs:1765-1774`: guarded by `!self.social.ghost_points.is_empty()`, `i0` and `i1` clamped to `pts.len() - 1`, denominator is constant `0.5` (no division by zero).
- **Blood Shrine generation & channeling**:
  - `crates/emberfall/src/main.rs:1064-1105`: channels at `dt` when `distance <= 68.0`, completes at `2.0s`, decays at `dt * 0.5` when outside ring, increments `shrine_score_mult += 0.5`, spawns elite `Golem`/`Wraith`, and triggers `Phase::LevelUp`.
  - `crates/emberfall/src/rng.rs:107-113`: `base_angle` is rolled *inside* `for i in 0..count`:
    `let base_angle = r.range_f32(0.0, std::f32::consts::TAU) + (i as f32) * std::f32::consts::PI;`
  - Empirical probe (`rustc` standalone binary running `generate_shrine_positions` across `2026-D001..2026-D100`) panicked:
    `thread 'main' panicked at probe.rs:128:13: Shrines too close: 31.45821`
- **Weapon Evolutions in `update_playing`**:
  - `CrimsonReaper` (`main.rs:1151-1180`): increases `search_r` (`320.0 -> 380.0`), `radius * 1.42`, `dmg * 1.48`, heals `+2.5` HP on hit.
  - `MeteorBarrage` (`main.rs:1192-1220`): `+2` projectiles, `vel` `560 -> 660`, `damage * 1.55`, `radius` `11 -> 16`, `pierce` `2 -> 4`.
  - `InfernoVortex` (`main.rs:1271-1284`, `1945-1968`): `spin_mult = 1.65`, `orbit_hit_cd` `0.2 -> 0.14`, `+2` orbs, `orb_r + 30.0`, `orb_dmg * 1.5`, hit radius `26 -> 34`.
- **`hitstop` & `trauma` decay**:
  - `main.rs:2601-2628`: `dt = get_frame_time().min(0.05)`; `hitstop` only skips `game.update_playing(dt)` while `poll_social_bridge()`, `M` mute, `P`/`Esc` pause, `draw_world`, and `draw_ui` continue executing. Max `hitstop` set in code is `0.08s`.
- **Shaders (`crates/emberfall/src/shaders.rs`)**:
  - `#version 100` with `precision mediump float;` in both fragment shaders; uniform names (`u_flash`, `u_time`, `u_low_hp`, `u_aberration`, `u_shockwave`, `u_blood_moon`) match `main.rs:1882` and `main.rs:2648-2655`.

## 2. Logic Chain
1. `rng::generate_shrine_positions` intends to place 2 shrines `PI` radians apart, as shown by `+ (i as f32) * PI` in `rng.rs:108`.
2. Because `r.range_f32(0.0, TAU)` is re-invoked inside the loop for `i = 1`, the two shrines have independent random angles and can land within `31.46` px of each other (empirically reproduced).
3. Since each Blood Shrine has a `64.0` px visual circle and `68.0` px channel radius (`main.rs:1069, 1694`), two shrines `< 136` px apart overlap visually and channel simultaneously, causing back-to-back elite spawns and overlapping `"BLOOD SHRINE"` text.
4. All other probed M2 mechanics (ghost bounds/interpolation, all 3 Weapon Evolutions, `hitstop`/`trauma` decay, FFI signature parity, native tests) passed inspection and empirical execution.

## 3. Caveats
- Probe directory `/tmp/teamwork_probe.wG6Y5q` was removed after execution; the working tree was not modified.
- `EvolutionId::accent` in `weapons.rs:92` emits a `dead_code` warning under `cargo test`.

## 4. Conclusion
`REQUEST_CHANGES`:
1. **Fix `rng::generate_shrine_positions` (`crates/emberfall/src/rng.rs:107-108`)**: Move `let base_angle = r.range_f32(0.0, std::f32::consts::TAU);` *outside* the `for i in 0..count` loop and compute `let angle = base_angle + (i as f32) * std::f32::consts::PI;` inside the loop so 2 shrines are guaranteed opposite (`>= 560` px apart).
2. *(Optional cleanup)* Remove or use `EvolutionId::accent` (`crates/emberfall/src/weapons.rs:92`) to clear the `dead_code` warning.

## 5. Verification Method
- Run `cargo test --manifest-path crates/emberfall/Cargo.toml` (expect 7/7 pass, 0 warnings if `accent` is addressed).
- Verify `generate_shrine_positions` separation `>= 560.0` px across `2026-D001..2026-D365` when `len() == 2`.
- Once `wasm32-unknown-unknown` is installed (M3), run `cargo check --manifest-path crates/emberfall/Cargo.toml --target wasm32-unknown-unknown`.

## Unchallenged Areas
- **`wasm32-unknown-unknown` compilation**: `rustup` in this environment does not have the `wasm32-unknown-unknown` target installed (`error[E0463]: can't find crate for core`), so `#[cfg(target_arch = "wasm32")]` compilation could not be executed.
- **Live WebGL / GLSL100 runtime compilation**: `shaders.rs` was verified by static inspection of `#version 100` syntax and uniform names against `main.rs`, not executed in a WebGL context.
