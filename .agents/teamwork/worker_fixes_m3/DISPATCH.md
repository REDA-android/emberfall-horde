## 2026-11-21T15:23:31Z

Sender: 5ccb6c664a1506539f23537429815fc5
Task: Fix all 3 Iteration 1 Gate findings + complete Milestone 3 (`README.md` update) for EMBERFALL: DAILY HORDE.

1. Fix Reviewer 2 finding (`crates/emberfall/src/main.rs`):
   - In `update_input()`, exclude top-right `mute_rect` and `pause_rect` HUD button hitboxes (matching `draw_ui()`'s exact coordinates) so tapping Pause or Mute during `Phase::Playing` does NOT activate `joy_active` / move the 1-thumb virtual joystick.
   - When transitioning to `Phase::Paused` (via keyboard `P`/`Esc` or touch HUD `||` button), clear `self.joy_active = false` and `self.joy_vec = Vec2::ZERO`.
2. Fix Challenger 2 findings (`crates/emberfall/src/rng.rs` + `crates/emberfall/src/weapons.rs` / `main.rs`):
   - In `crates/emberfall/src/rng.rs` (`generate_shrine_positions`), move `let base_angle = r.range_f32(0.0, std::f32::consts::TAU);` OUTSIDE the `for i in 0..count` loop so 2 shrines are guaranteed to spawn ~`PI` radians apart. Add unit test asserting minimum distance `>= 500.0` px between shrines when `count == 2`.
   - Eliminate `dead_code` warning on `EvolutionId::accent` in `crates/emberfall/src/weapons.rs` by using `evo.accent()` when setting `evo_banner_color` in `crates/emberfall/src/main.rs` (zero compiler warnings!).
3. Fix Challenger 1 finding (`dist/client/splash.html`):
   - Update `hashSeed(seed)` in `dist/client/splash.html` to use `BigInt` (`0xcbf29ce484222325n`, `0x100000001b3n`, `0x9e3779b97f4a7c15n`, `0x2545f4914f6cdd1dn`, masked to `0xffffffffffffffffn`) matching `src/server/index.ts` and `crates/emberfall/src/rng.rs` identically.
4. Update `README.md` (Milestone 3).
5. Verify: `cargo check`, `cargo test`, `npm run typecheck`, `npm run build:server`, CSP check, write `handoff.md`, message caller.
