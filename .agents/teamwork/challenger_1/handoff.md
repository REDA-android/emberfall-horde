# Challenger 1 Handoff Report — Server, Bridge & FFI Contract Adversarial Verification

**Verdict**: `APPROVE`

---

## 1. Observation

All probes were executed against the repository at `/home/antigravity/REDA-android/emberfall-horde` using a temporary directory (`/tmp/teamwork_probe.HTpsfb`, `chmod 700`), which was removed (`rm -rf`) before reporting.

### 1.1 TypeScript Check & Artifact Inventory
- `npm run typecheck` (`tsc --noEmit`): exited `0` with zero errors.
- `dist/client/splash.html`: `11,985` bytes.
- `dist/client/index.html`: `17,681` bytes.
- `dist/client/emberfall.wasm`: `952,858` bytes (mtime `Nov 21 14:52`).
- `dist/server/index.cjs`: `3,073,092` bytes (mtime `Nov 21 15:21`).

### 1.2 `daySeed` / `hashSeed` / `mutatorIndex` Parity
Compiled `crates/emberfall/src/rng.rs` via `rustc` and compared `DailyModifier::from_seed` against `defaultMutatorFromSeed` extracted from `src/server/index.ts:44-58` and `defaultMutator` extracted from `dist/client/splash.html:282-300` across 38 seeds (`2026-11-21`, `2026-01-01`, `2026-12-31`, `2026-02-28`, `2024-02-29`, `1970-01-01`, `2027-01-01`, `""`, and `2026-D001`..`2026-D030`):
- Result: `Tested 38 seeds across Rust (rng.rs), Server (index.ts), and Splash (splash.html). Mismatches: 0`
- Verified `crates/emberfall/src/main.rs:378-383` formats `emberfall_get_day_number()` (`YYYYMMDD`) into `"{:04}-{:02}-{:02}"` matching `getUtcDateString()` in `src/server/index.ts:36-42` and `utcSeed()` in `dist/client/splash.html:274-280`.

### 1.3 FFI Symbol Name & Signature Parity (`main.rs` <-> `index.html`)
Parsed `#[cfg(target_arch = "wasm32")] extern "C"` (`crates/emberfall/src/main.rs:12-52`) and `importObject.env` (`dist/client/index.html:241-454`):
- `Rust extern "C" count: 16, index.html env.* count: 17`
- All 16 Rust `extern "C"` functions matched `index.html` in exact name and parameter count:
  - `emberfall_get_day_number` (0 args)
  - `emberfall_play_sfx_pitched` (2 args)
  - `emberfall_set_muted` (1 arg)
  - `emberfall_toggle_drone` (1 arg)
  - `emberfall_on_run_start` (0 args)
  - `emberfall_on_run_complete` (8 args)
  - `emberfall_share_comment` (6 args)
  - `emberfall_vote_mutator` (1 arg)
  - `emberfall_poll_init_ready` (0 args)
  - `emberfall_get_init_field` (1 arg)
  - `emberfall_copy_str_field` (4 args)
  - `emberfall_get_lb_score` (1 arg)
  - `emberfall_get_ghost_x` (1 arg)
  - `emberfall_get_ghost_y` (1 arg)
  - `emberfall_get_share_state` (0 args)
  - `emberfall_get_submit_state` (0 args)
- `index.html:249-251` also defines `env.emberfall_play_sfx` (1 arg) as a backwards-compatible alias.
- **Note on pre-M2 `dist/client/emberfall.wasm`**: `WebAssembly.Module.imports` on the existing `dist/client/emberfall.wasm` (built at `14:52`) reports only 4 `emberfall_*` imports (`emberfall_get_day_number`, `emberfall_play_sfx`, `emberfall_on_run_complete`, `emberfall_share_comment`), confirming it has not yet been rebuilt for M2 (scheduled in M3 per `PROJECT.md:48,55`).

### 1.4 Server Anti-Cheat & Auth Edge Cases (`src/server/index.ts`)
Transpiled `src/server/index.ts` to CJS via `typescript` and ran live HTTP requests against it with an in-memory `@devvit/web/server` mock:
1. `POST /api/submit-run` with `runId` elapsed `5s` (`< 10s`): `400 { ok: false, error: 'Run duration out of bounds' }` (`src/server/index.ts:415-418`).
2. `POST /api/submit-run` with `runId` elapsed `185s` (`> 180s`): `400 { ok: false, error: 'Run duration out of bounds' }` (`src/server/index.ts:415-418`).
3. `POST /api/submit-run` with `runId` elapsed `45s`, `score: -500`, `kills: -10`, `level: -3`, and `250` mixed `ghostPoints` (`['bad', null]`, `'notarray'`, `[10]`, `[i, -i]`):
   - Returned `200 { ok: true, rank: 'E', myRank: 1, ... }`.
   - Leaderboard score clamped to `0` (`src/server/index.ts:436`).
   - Stored ghost truncated to `<= 200` (`198` valid array entries) and non-numeric `['bad', null]` sanitized to `[0, 0]` (`src/server/index.ts:472-482`).
4. Rate limit (2nd `POST /api/submit-run` within 10s): `429 { ok: false, error: 'Too many submissions, please wait 10s' }` (`src/server/index.ts:427-434`).
5. **Adversarial Finding (Medium)**: `POST /api/submit-run` **without `runId`** (`{ score: 500000, kills: 9999, bossSlain: true, level: 50 }`) returned `200 { ok: true, rank: 'SSS', myRank: 1, totalPlayers: 1, streak: 1, bossKillsToday: 1, isNewBest: true, canVoteMutator: true }`. Because `src/server/index.ts:406` gates session verification on `if (typeof body.runId === 'string' && body.runId.length > 0)`, omitting `runId` bypasses the `10s..180s` duration check completely. (This matches `PROJECT.md:82` `"Validates runId (if present, elapsed 10s..180s)"`, so it is compliant with the M1 spec, but represents an anti-cheat hardening opportunity.)
6. `POST /api/share-comment` unauthenticated (`username === undefined`): `401 { ok: false, error: 'Sign in to Reddit to post your score' }` (`src/server/index.ts:580-583`).
7. `POST /api/share-comment` authenticated, then repeated with same score: `200 { ok: true }` followed by `429 { ok: false, error: 'Already shared this score today' }` (`src/server/index.ts:607-612`).
8. `POST /api/vote-mutator` as rank #4 and as unranked user: `403 { ok: false, error: 'Top 3 daily rank required to vote' }` (`src/server/index.ts:544-547`). As rank #2 with `mutatorIndex: 2`: `200 { ok: true, mutatorIndex: 2 }`. With `mutatorIndex: 4`: `400 { ok: false, error: 'mutatorIndex must be 0..3' }`.

### 1.5 Strict CSP & Syntax Compliance
- `dist/client/splash.html`: `11,985` bytes (`< 30,720`), `0` `http://`/`https://` URLs, `0` external `<script src>`, inline `<script>` passes `node --check`.
- `dist/client/index.html`: `17,681` bytes (`< 30,720`), `0` `http://`/`https://` URLs, only local `<script src="./mq_js_bundle.js">`, inline `<script>` passes `node --check`.

---

## 2. Logic Chain

1. Observation 1.2 confirms that `src/server/index.ts`, `dist/client/splash.html`, and `crates/emberfall/src/rng.rs` + `main.rs` share identical `YYYY-MM-DD` UTC seed formatting and identical FNV-1a + `xorshift64*` mutator selection across all tested dates.
2. Observation 1.3 confirms 100% symbol-name and parameter-arity parity between `crates/emberfall/src/main.rs` (`extern "C"`) and `dist/client/index.html` (`importObject.env`) for all 16 M2 FFI functions.
3. Observation 1.4 confirms all server-side bounds clamping, rate limiting, ghost sanitization, authentication, and top-3 mutator-voting guards behave as specified in `PROJECT.md`, while surfacing the `runId`-omission bypass as a non-blocking Medium advisory.
4. Observation 1.5 confirms strict CSP compliance and valid JS syntax for both client HTML entrypoints.
5. Therefore, M1 + FFI contract parity meets the `PROJECT.md` specification and is ready for M3 integration (`build:wasm` + `build:server`).

---

## 3. Caveats

- Server HTTP probes used an in-memory mock of `@devvit/web/server` (`redis`, `reddit`, `context`) rather than live Reddit/Devvit infrastructure.
- `dist/client/emberfall.wasm` on disk is pre-M2 (4 `emberfall_*` imports) and must be rebuilt in M3 (`npm run build:wasm`) to exercise the 16 M2 FFI symbols in-browser.
- Advisory hardening note: in M3 or post-hackathon, consider requiring `runId` (or gating `#1` ghost / `canVoteMutator` on a verified `runId`) so attackers cannot bypass `10s..180s` duration validation by omitting `runId`.

## Unchallenged Areas

- Live `devvit playtest` / Reddit mobile WebView runtime behavior (WebAudio unlock on iOS/Android Reddit app, `requestExpandedMode` in live feed).
- Runtime execution of the M2 `emberfall.wasm` binary inside `dist/client/index.html` (pending M3 `cargo build --release --target wasm32-unknown-unknown`).
- `src/server/index.ts` scheduler (`/internal/scheduler/daily-post`) and `onPostDelete` Redis key cleanup under concurrent Devvit invocations.

---

## 4. Conclusion

**Verdict: `APPROVE`**

M1 (`src/server/index.ts`, `devvit.json`, `dist/client/splash.html`, `dist/client/index.html`) and the Rust/JS FFI contract (`crates/emberfall/src/main.rs`, `rng.rs` <-> `dist/client/index.html`) pass all empirical parity, anti-cheat, auth, and strict CSP checks. Proceed to M3 WASM rebuild (`npm run build`) and full integration verification.

---

## 5. Verification Method

To independently verify:
1. Run `npm run typecheck` in `/home/antigravity/REDA-android/emberfall-horde` (expected exit `0`).
2. Check client sizes and external URLs:
   ```bash
   wc -c dist/client/splash.html dist/client/index.html
   rg -n 'https?://' dist/client/splash.html dist/client/index.html
   ```
   (Expected: both `< 30720` bytes, `rg` exits `1` / zero matches.)
3. Verify `rng.rs` unit tests:
   ```bash
   cd crates/emberfall && cargo test --lib rng
   ```
4. Verify `crates/emberfall/src/main.rs` lines `12-52` against `dist/client/index.html` lines `241-454` for exact 16-symbol FFI parity.
