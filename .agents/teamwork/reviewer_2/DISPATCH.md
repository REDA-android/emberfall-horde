## 2026-11-21T15:21:04Z

You are Reviewer 2 (Secondary Reviewer — Shaders, Devvit Compliance & Mobile UX) for EMBERFALL: DAILY HORDE (M1 + M2 Verification Wave).

MANDATORY FIRST STEP: Read `/home/antigravity/REDA-android/emberfall-horde/.agents/teamwork/ORIGINAL_REQUEST.md` verbatim before anything else.
Then read `/home/antigravity/REDA-android/emberfall-horde/PROJECT.md`, `/home/antigravity/REDA-android/emberfall-horde/.agents/teamwork/worker_m1/handoff.md`, and `/home/antigravity/REDA-android/emberfall-horde/.agents/teamwork/worker_m2/handoff.md`.

Your state folder: `/home/antigravity/REDA-android/emberfall-horde/.agents/teamwork/reviewer_2`
Project root: `/home/antigravity/REDA-android/emberfall-horde`

Rules:
- MUST NOT modify the working tree or put source, probes/tests, or data in `.agents/teamwork/`.
- Run only target-scoped checks (do NOT run `npm run build:server` or full `cargo test` concurrently with Reviewer 1 — focus on code inspection + `cargo check --manifest-path crates/emberfall/Cargo.toml` if needed, timestamping in `progress.md`).

Tasks:
1. Review `crates/emberfall/src/shaders.rs` and `main.rs` shader usage:
   - Verify `#version 100` and `precision mediump float;` in all fragment/vertex shaders.
   - Verify all `UniformDesc` names/types in `shaders.rs` match the GLSL uniforms AND `set_uniform(...)` calls in `main.rs`!
   - Verify `render_target` `flip_y` / camera orientation and `gl_use_default_material()` restoration so HUD/UI is never corrupted.
2. Review `devvit.json`, `src/server/index.ts`, `dist/client/splash.html`, `dist/client/index.html`:
   - Devvit compliance: `default` (`splash.html`) + `game` (`index.html`), `requestExpandedMode`, `scheduler.tasks` with `SET NX` lock, `onPostDelete` cleanup, 30-day `redis.expire` TTLs, `runAs: 'USER'` on `submitComment`, strict CSP.
   - Mobile UX: iOS WebAudio synchronous unlock, 1-thumb joystick vs Pause/Mute HUD button hitboxes (ensure Pause/Mute buttons don't accidentally block or trigger dash/joystick).

Write your structured verdict (`APPROVE` or `REQUEST_CHANGES`) with exact file/line evidence to `/home/antigravity/REDA-android/emberfall-horde/.agents/teamwork/reviewer_2/handoff.md`, and message the caller.
