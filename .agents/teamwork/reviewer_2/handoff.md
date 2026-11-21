# Reviewer 2 Handoff Report — Shaders, Devvit Compliance & Mobile UX

## Review Summary

**Verdict**: `REQUEST_CHANGES`

Shaders (`crates/emberfall/src/shaders.rs` + `main.rs`), Devvit compliance (`devvit.json`, `src/server/index.ts`, `dist/client/splash.html`, `dist/client/index.html`), and iOS WebAudio synchronous unlock are solid and verified. However, there is **1 Major Mobile UX defect** in `crates/emberfall/src/main.rs`: tapping the top-right **Pause** or **Mute** HUD buttons during `Phase::Playing` is not excluded from `update_input()`, causing those taps to activate the 1-thumb virtual joystick at the top-right corner of the screen.

---

## 1. Observation

### Task 1: Shaders (`crates/emberfall/src/shaders.rs` & `crates/emberfall/src/main.rs`)
1. **GLSL ES 1.00 Directives (`crates/emberfall/src/shaders.rs`)**:
   - `VERTEX_SHADER` (`shaders.rs:3-19`): `#version 100` on line 3; explicit `lowp` on `varying`s.
   - `HIT_FLASH_FRAGMENT` (`shaders.rs:21-35`): `#version 100` (line 21), `precision mediump float;` (line 22).
   - `POST_PROCESS_FRAGMENT` (`shaders.rs:37-93`): `#version 100` (line 37), `precision mediump float;` (line 38).
2. **Uniform Name & Type Parity**:
   - `hit_flash`: GLSL `uniform float u_flash;` (`shaders.rs:28`) == `UniformDesc::new("u_flash", UniformType::Float1)` (`shaders.rs:108`) == `mat.set_uniform("u_flash", (e.hit_flash / 0.12).clamp(0.0, 1.0))` (`main.rs:1882`, `f32`).
   - `post_process`: GLSL `u_time`, `u_low_hp`, `u_aberration`, `u_shockwave`, `u_blood_moon` (`shaders.rs:44-48`, all `float`) == `UniformDesc` entries (`shaders.rs:121-125`, all `UniformType::Float1`) == `mat.set_uniform(...)` calls (`main.rs:2648-2655`, all `f32`).
3. **`render_target`, `flip_y`, and `gl_use_default_material()` Restoration**:
   - `draw_world()` (`main.rs:1625-1630`) renders into `rt` with `Camera2D { zoom: vec2(2.0 / sw, -2.0 / sh), render_target: rt.cloned(), .. }`, then calls `set_default_camera()` (`main.rs:2051`).
   - Hit-flash material (`main.rs:1880-1899`) calls `gl_use_default_material()` immediately after each flashed enemy's `draw_texture_ex`.
   - Main loop (`main.rs:2630-2671`) resizes `rt` if `(sw as u32, sh as u32)` changes, draws `&rt.texture` with `flip_y: true` (`main.rs:2665`), calls `gl_use_default_material()` (`main.rs:2669`), and then calls `game.draw_ui(&assets)` (`main.rs:2671`).
4. **Target-Scoped Check**:
   - `cargo check --manifest-path crates/emberfall/Cargo.toml` exited `0` (1 dead_code warning on `EvolutionId::accent` at `src/weapons.rs:92`).

### Task 2: Devvit Compliance & Mobile UX
1. **Devvit Compliance (`devvit.json`, `src/server/index.ts`, `dist/client/splash.html`, `dist/client/index.html`)**:
   - `devvit.json:7-13`: `post.entrypoints.default` -> `splash.html` (`height: "tall"`), `post.entrypoints.game` -> `index.html`.
   - `dist/client/splash.html:308-338`: `12K` (`< 30 KB`), zero WASM, calls `globalThis.requestExpandedMode(event, 'game')` with Devvit `immersiveMode` `postMessage` fallback and `./index.html` fallback.
   - `devvit.json:28-36` & `src/server/index.ts:159-176, 235-239`: `scheduler.tasks.daily-post` (`0 * * * *`) -> `/internal/scheduler/daily-post` -> `createDailyPost({ respectLock: true })` using `redis.set(lockKey(...), '1', { nx: true, expiration: ... })`.
   - `devvit.json:29` & `src/server/index.ts:250-287`: `triggers.onPostDelete` -> `/internal/on-post-delete` purges `postKey`, `lockKey`, `postIndexKey`, `shareGuardKey`s, and `postSharesKey`.
   - `src/server/index.ts:27, 186, 188, 447, 462, 466, 486, 505, 551, 629, 631`: 30-day `redis.expire(..., THIRTY_DAYS_SEC)` (`30 * 86400`) on all daily/user-identifying keys.
   - `devvit.json:24` & `src/server/index.ts:623-627`: `asUser: ["SUBMIT_COMMENT"]` and `reddit.submitComment({ id: context.postId, text, runAs: 'USER' })`.
   - Strict CSP: `! grep -nE 'https?://' dist/client/splash.html dist/client/index.html` -> `CSP_CLEAN` (`12K` and `18K`).
2. **Mobile UX (`dist/client/index.html` & `crates/emberfall/src/main.rs`)**:
   - `dist/client/index.html:21-48`: `unlockAudio()` is bound to `pointerdown`, `touchstart`, and `keydown`, synchronously calling `ensureAudioCtx()` (`AudioContext`/`webkitAudioContext` + `resume()`) and playing a 1-sample silent `AudioBufferSourceNode` via `src.start(0)`.
   - `crates/emberfall/src/main.rs:920-974` vs `2249-2271`: `update_input()` excludes `dash_center` (`dash_r + 14.0` touch / `dash_r + 10.0` mouse), **but does NOT exclude `mute_rect` (`Rect::new(sw - 84.0, 16.0, 32.0, 28.0)`) or `pause_rect` (`Rect::new(sw - 46.0, 16.0, 32.0, 28.0)`)**.

---

## 2. Logic Chain

1. In `main.rs:2626`, `game.update_playing(dt)` runs **before** `game.draw_ui(&assets)` at `main.rs:2671`.
2. During `Phase::Playing`, when a user taps or clicks `mute_rect` or `pause_rect` in the top-right HUD, `update_input()` (`main.rs:928-974`) sees a `TouchPhase::Started` (or `MouseButton::Left`) that is not near `dash_center`, so it sets `self.joy_active = true` and `self.joy_origin = pos` at the HUD button position.
3. Subsequently in `draw_ui()` (`main.rs:2251-2270`), `draw_button(mute_rect, ...)` or `draw_button(pause_rect, ...)` also triggers, toggling mute or transitioning to `Phase::Paused` without clearing `self.joy_active` / `self.joy_vec`.
4. Therefore, Pause/Mute taps accidentally trigger the virtual joystick (and `pause_rect`/`mute_rect` at `32x28` are also tight for mobile thumbs).

---

## 3. Findings & Challenges

### [Major] Finding 1: Pause and Mute HUD button taps trigger the 1-thumb virtual joystick
- **What**: Tapping or clicking `mute_rect` or `pause_rect` during `Phase::Playing` activates `self.joy_active = true` at the top-right HUD coordinates, and pausing does not reset `joy_active`/`joy_vec`.
- **Where**: `crates/emberfall/src/main.rs:928-948` (`update_input`) and `crates/emberfall/src/main.rs:2249-2271` (`draw_ui`).
- **Why**: `update_input()` only checks `dash_center` before assigning `active_pointer` to the virtual joystick, and runs before `draw_ui()` handles `mute_rect` / `pause_rect`.
- **Suggestion**:
  1. Extract shared `hud_mute_rect(sw)` and `hud_pause_rect(sw)` helpers (and consider slightly larger hitboxes or `+6px` touch padding).
  2. In `update_input()`, skip any touch/mouse pointer inside `mute_rect` or `pause_rect` (so it neither triggers Dash nor activates `active_pointer` / `joy_active`).
  3. Clear `self.joy_active = false; self.joy_vec = Vec2::ZERO;` when entering `Phase::Paused`.

### [Minor] Finding 2: Dead-code compiler warning in `weapons.rs`
- **What**: `warning: method 'accent' is never used` on `EvolutionId::accent` (`crates/emberfall/src/weapons.rs:92`).
- **Suggestion**: Use `evo.accent()` when setting `self.evo_banner_color` at `main.rs:1565` or mark `#[allow(dead_code)]`.

---

## 4. Caveats
- `wasm32-unknown-unknown` build and live `devvit playtest` / iOS Safari WebView runtime were not executed in this wave (per Reviewer 2 scope).
- `npm run build:server` and `cargo test` were intentionally not run to avoid concurrent workspace contention with Reviewer 1.

---

## 5. Conclusion

**Verdict: `REQUEST_CHANGES`**

Shaders (`#version 100`, `precision mediump float;`, uniform parity, `render_target` `flip_y`, `gl_use_default_material()`) and Devvit compliance (`default`/`game` entrypoints, `requestExpandedMode`, `SET NX` lock, `onPostDelete`, 30-day TTLs, `runAs: 'USER'`, strict CSP, iOS WebAudio unlock) all **PASS**. Fix **Finding 1** in `crates/emberfall/src/main.rs` so Pause/Mute HUD buttons are excluded from `update_input()` and reset joystick state on pause.

---

## 6. Verification Method
Run in `/home/antigravity/REDA-android/emberfall-horde`:
```bash
cargo check --manifest-path crates/emberfall/Cargo.toml
! grep -nE 'https?://' dist/client/splash.html dist/client/index.html && echo "CSP_CLEAN"
```
Inspect `crates/emberfall/src/main.rs` (`update_input` around lines 920-975 and `draw_ui` around lines 2248-2271) to confirm `mute_rect` and `pause_rect` are excluded from joystick activation.
