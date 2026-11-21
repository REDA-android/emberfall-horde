# Handoff Report — Fix & Milestone 3 Integration Worker

## 1. Observation

- **`crates/emberfall/src/main.rs`**:
  - `update_input()` (`902–989`): Previously only checked `dash_center` / `dash_r` before activating `self.joy_active`. Added `pause_rect = Rect::new(sw - 46.0, 16.0, 32.0, 28.0)` and `mute_rect = Rect::new(sw - 84.0, 16.0, 32.0, 28.0)` matching `draw_ui()` (`2255–2256`), and excluded them when `!self.joy_active` for both touch (`touches()`) and mouse (`MouseButton::Left`).
  - `draw_ui()` (`2274–2276`) and main loop keyboard `P`/`Esc` (`2619–2621`): Added `self.joy_active = false; self.joy_vec = Vec2::ZERO;` (and `game.joy_active = false; game.joy_vec = Vec2::ZERO;`) when transitioning to `Phase::Paused`.
  - Evolution unlock (`1573`): Added `self.evo_banner_color = evo.accent();` inside `if let Some(evo) = self.build.newly_unlocked_evolution(&prev)`.
- **`crates/emberfall/src/rng.rs`**:
  - `generate_shrine_positions()` (`103–116`): Moved `let base_angle = r.range_f32(0.0, std::f32::consts::TAU);` outside `for i in 0..count` and computed `let angle = base_angle + (i as f32) * std::f32::consts::PI;`.
  - `deterministic_shrines_per_seed()` (`203–209`): Added assertion checking Euclidean distance `sep >= 500.0` between the two shrines whenever `pts.len() == 2`.
- **`dist/client/splash.html`**:
  - Inspected `defaultMutator(seed)` (`282–300`): Already uses `BigInt` (`0xcbf29ce484222325n`, `0x100000001b3n`, `0x9e3779b97f4a7c15n`, `0x2545f4914f6cdd1dn`, masked with `0xffffffffffffffffn`), matching `src/server/index.ts:44–58` and `crates/emberfall/src/rng.rs:8–27` identically. No edit required.
- **`README.md`**:
  - Updated (`1–76`) to document the Devvit Web + Redis architecture (`splash.html` + `requestExpandedMode`, `index.html`, `scheduler.tasks`, `onPostDelete`, 30-day TTLs, `#1 Champion Ghost Replay`, `Subreddit Boss Siege`, `Winner-Picks-Tomorrow's-Mutator`, `Wordle-style Emoji Build Share Card`), pure-Rust features (GLSL100 shaders, hitstop/trauma shake, 3 Weapon Evolutions, Blood Shrines, Kill-Streak Fever Meter, Pause/Mute), and exact build/test commands.
- **Verification Command Outputs**:
  - `cargo check --manifest-path crates/emberfall/Cargo.toml`: exited `0` with **0 warnings** (`Finished dev profile ... in 0.19s`).
  - `cargo test --manifest-path crates/emberfall/Cargo.toml`: exited `0`, `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`.
  - `npm run typecheck`: exited `0` (`tsc --noEmit`).
  - `npm run build:server`: exited `0` (`dist/server/index.cjs  3,073.06 kB`, `built in 3.46s`).
  - `! grep -nE 'https?://' dist/client/splash.html dist/client/index.html && echo "CSP_CHECK_PASSED"`: exited `0`, printed `CSP_CHECK_PASSED`.
  - `npm run build:wasm`: exited `101` (`error[E0463]: can't find crate for std` — `wasm32-unknown-unknown` target is not installed in this VM).

## 2. Logic Chain

1. Excluding `mute_rect` and `pause_rect` in `update_input()` with the exact coordinates from `draw_ui()` prevents taps/clicks on those HUD buttons from initiating `joy_active`, while clearing `joy_active` and `joy_vec` on both HUD and keyboard pause transitions prevents stale joystick state upon pausing or resuming.
2. Moving `base_angle` outside the loop in `generate_shrine_positions()` ensures that when `count == 2`, the two shrines are placed `PI` radians apart at `dist >= 280.0` px each, guaranteeing a minimum separation of `560.0 >= 500.0` px, verified by the new assertion in `deterministic_shrines_per_seed`.
3. Assigning `self.evo_banner_color = evo.accent()` uses `EvolutionId::accent` in `weapons.rs`, eliminating the sole compiler warning (`cargo check` now produces 0 warnings).
4. `dist/client/splash.html` already matched `src/server/index.ts` and `crates/emberfall/src/rng.rs` with `BigInt` FNV-1a + `xorshift64*`.
5. `README.md` now accurately reflects the M1–M3 architecture, features, controls, and build/test commands.

## 3. Caveats

- `npm run build:wasm` could not complete in this VM because the `wasm32-unknown-unknown` Rust target is not installed (`rustup target add wasm32-unknown-unknown` is required to regenerate `dist/client/emberfall.wasm` with the M2 + Iteration 1 fixes).

## 4. Conclusion

All 3 Iteration 1 Gate findings are resolved (`crates/emberfall/src/main.rs`, `crates/emberfall/src/rng.rs`, and `dist/client/splash.html` verified) and Milestone 3 `README.md` is updated. Native Rust check (`0` warnings), all `7/7` Rust unit tests, TypeScript typecheck, server bundle build, and strict CSP check all pass.

## 5. Verification Method

Run from `/home/antigravity/REDA-android/emberfall-horde`:
```bash
cargo check --manifest-path crates/emberfall/Cargo.toml
cargo test --manifest-path crates/emberfall/Cargo.toml
npm run typecheck
npm run build:server
! grep -nE 'https?://' dist/client/splash.html dist/client/index.html && echo "CSP_CHECK_PASSED"
```
(To rebuild `dist/client/emberfall.wasm` on a host with `wasm32-unknown-unknown` installed: `npm run build:wasm`.)
