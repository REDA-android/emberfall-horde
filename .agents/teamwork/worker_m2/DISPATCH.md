## 2026-11-21T15:14:11Z

Sender: `5ccb6c664a1506539f23537429815fc5` (parent)

Task: Milestone 2 Worker (Pure-Rust Juice, Shaders, Ghost Replay & 90s Depth) for EMBERFALL: DAILY HORDE.

Exclusive file ownership (do NOT touch `src/server/*` or `dist/client/*`):
- `crates/emberfall/src/shaders.rs` (new)
- `crates/emberfall/src/rng.rs`
- `crates/emberfall/src/weapons.rs`
- `crates/emberfall/src/main.rs`

Tasks:
1. FFI Bridge & Social HUD (`crates/emberfall/src/main.rs`)
2. #1 Champion Ghost Replay (`crates/emberfall/src/main.rs`)
3. GLSL100 Shaders (`crates/emberfall/src/shaders.rs` + `main.rs`)
4. Game Feel ("Juice"), Pause & Mute (`crates/emberfall/src/main.rs`)
5. 90-Second Survivor Depth (`crates/emberfall/src/weapons.rs`, `rng.rs`, `main.rs`)
6. Unit Tests (`rng.rs`, `weapons.rs`)

Verification:
- `cargo check --manifest-path crates/emberfall/Cargo.toml`
- `cargo test --manifest-path crates/emberfall/Cargo.toml`
- `cargo check --manifest-path crates/emberfall/Cargo.toml --target wasm32-unknown-unknown` (if installed)
- Write `handoff.md` in `/home/antigravity/REDA-android/emberfall-horde/.agents/teamwork/worker_m2/handoff.md` and message caller.
