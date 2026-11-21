# Milestone 1 Handoff Report: Devvit Compliance, Server & Splash/Audio Bridge

## 1. Observation

1. **Toolchain (`rustup` / `wasm32`):**
   - `which rustup cargo node npm` returned `/usr/local/bin/rustup`, `/usr/local/bin/cargo`, `/usr/local/bin/node`, `/usr/local/bin/npm`.
   - `rustup target list --installed` returned only `x86_64-unknown-linux-gnu`.
   - Running `rustup target add wasm32-unknown-unknown` exited with code `1`:
     `error: component download failed for rust-std-wasm32-unknown-unknown: error opening file for download: cleaning up cached downloads: Read-only file system (os error 30)`.
2. **TypeScript & Package Config (`tsconfig.json`, `package.json`):**
   - Created `tsconfig.json` targeting `ES2022`, `module: "NodeNext"`, `moduleResolution: "NodeNext"`, `strict: true`, `skipLibCheck: true`, `noEmit: true`, `include: ["src/server/**/*.ts"]`.
   - Added `"typecheck": "tsc --noEmit"` to `package.json`.
3. **Devvit Manifest (`devvit.json`):**
   - Split `post.entrypoints` into `default` (`{ "entry": "splash.html", "height": "tall" }`) and `game` (`{ "entry": "index.html" }`).
   - Added `triggers.onPostDelete: "/internal/on-post-delete"` and `scheduler.tasks.daily-post` (`cron: "0 * * * *"`, `endpoint: "/internal/scheduler/daily-post"`).
4. **Server & Redis (`src/server/index.ts`):**
   - Implemented `/internal/scheduler/daily-post` using atomic `redis.set(lockKey, '1', { nx: true, expiration: new Date(Date.now() + 600_000) })` + `ef:post:<sub >:<seed>` check so each UTC day posts once idempotently with the winning daily mutator in the post title.
   - Implemented `/internal/on-post-delete` to purge post-scoped keys (`ef:postidx:<postId>`, matching `ef:post:<sub >:<seed>` / `ef:lock:<sub >:<seed>`, and `ef:shared:<postId>:<seed>:<user>` tracked via `ef:post-shares:<postId>`).
   - Added `redis.expire(..., 30 * 86400)` TTLs to daily user-identifying keys (`ef:lb:*`, `ef:meta:*`, `ef:streak:*`, `ef:bosses:*`, `ef:ghost:*`, `ef:mutator-vote:*`, `ef:shared:*`, `ef:post:*`, `ef:postidx:*`, `ef:post-shares:*`).
   - Implemented `POST /api/start-run` returning `{ ok: true, runId, startedAt }` stored in `ef:run:<runId>` with 300s TTL.
   - Upgraded `POST /api/submit-run` to validate `runId` (elapsed `10_000ms..180_000ms`), rate-limit per user (`ef:rate:<sub >:<user>`, 10s `NX` lock), clamp bounds, update `ef:lb`, `ef:meta`, `ef:streak`, `ef:bosses`, store #1 Champion Ghost (`ef:ghost:<sub >:<seed>`, up to 200 `[x,y]` pairs), and return `{ ok, rank, myRank, totalPlayers, streak, bossKillsToday, isNewBest, canVoteMutator }`.
   - Upgraded `GET /api/init` to match `PROJECT.md § Interface Contracts` (`seed`, `mutatorOverride`, `username`, `streak`, `personalBest`, `myRank`, `totalPlayers`, `bossKillsToday`, `siegeTarget`, `leaderboard`, `topGhost`, `canVoteMutator`).
   - Implemented `POST /api/vote-mutator` (`mutatorIndex: 0..3`) restricted to top-3 daily players (`403` otherwise), stored in `ef:mutator-vote:<sub >:<tomorrowSeed>` and applied in tomorrow's `/api/init` and daily post title.
   - Upgraded `POST /api/share-comment` to post a Wordle-style Markdown + Emoji Build Share Card as `USER`, returning `401` when unauthenticated, `429` if already shared at `>= score`, and `200` on success.
5. **Inline Splash (`dist/client/splash.html`):**
   - Created `12K` (`< 30 KB`) self-contained zero-WASM, zero-external-URL dark-fantasy card fetching `/api/init` and rendering Dungeon `#seed`, Daily Mutator, Subreddit Boss Siege bar, Top 3 Leaderboard, and **"⚔️ ENTER DUNGEON"** calling `requestExpandedMode(event, 'game')` (with Devvit `immersiveMode` `postMessage` and `./index.html` fallback).
6. **Expanded Game Shell, WebAudio & FFI Bridge (`dist/client/index.html`):**
   - Added synchronous `pointerdown`/`touchstart`/`keydown` `AudioContext` unlocker, `emberfall_play_sfx_pitched(kind, pitch_mul)`, `emberfall_play_sfx(kind)`, `emberfall_set_muted(muted)`, and `emberfall_toggle_drone(active)` (low-pass `55Hz`/`82.41Hz` dark-fantasy drone).
   - Implemented all `emberfall_bridge` `miniquad_add_plugin` functions from `PROJECT.md § Interface Contracts`: `emberfall_get_day_number`, `emberfall_on_run_start`, `emberfall_on_run_complete`, `emberfall_share_comment`, `emberfall_vote_mutator`, `emberfall_poll_init_ready`, `emberfall_get_init_field`, `emberfall_copy_str_field`, `emberfall_get_lb_score`, `emberfall_get_ghost_x`, `emberfall_get_ghost_y`, `emberfall_get_share_state`, `emberfall_get_submit_state`.
   - `submit_state` and `share_state` properly transition through `1` (pending), `2` (ok), and `3` (error) with `lastStatusMsg`.

## 2. Logic Chain

1. **Seed & Mutator Parity:** Inspecting `crates/emberfall/src/main.rs:220` and `rng.rs:8-62` showed `day_seed` is `YYYY-MM-DD` hashed with FNV-1a + `xorshift64*`. Implementing the identical `BigInt` hash in `src/server/index.ts` and `dist/client/splash.html` guarantees server, splash, and Rust agree on the daily mutator.
2. **Devvit `requestExpandedMode` in Static HTML:** Inspecting `@devvit/client/effects/web-view-mode.js` and `@devvit/shared-types/client/emit-effect.js` confirmed `requestExpandedMode(event, 'game')` posts `{ type: 'devvit-internal', scope: 0, immersiveMode: { entryUrl, immersiveMode: 2 } }` to `parent`. Supporting `globalThis.requestExpandedMode`, the `devvit.entrypoints.game` `postMessage` protocol, and `./index.html` fallback keeps `splash.html` `< 30 KB` with zero external URLs.
3. **FFI Memory Safety:** Inspecting `dist/client/mq_js_bundle.js` confirmed `wasm_memory` and `UTF8ToString` are globals. Reading `build_ptr`/`ghost_ptr` synchronously before `fetch()` prevents stale pointer reads across async boundaries.

## 3. Caveats

- `rustup target add wasm32-unknown-unknown` could not complete in this sandbox because `rustup`'s download cache directory is on a read-only filesystem (`os error 30`). Rebuilding `emberfall.wasm` requires a writable `RUSTUP_HOME` or an environment with `wasm32-unknown-unknown` pre-installed.
- Runtime HTTP/Redis behavior and `requestExpandedMode` in Reddit's WebView host were verified statically (`tsc --noEmit`, `vite build`, CSP grep) rather than against a live `devvit playtest` subreddit.

## 4. Conclusion

Milestone 1 is complete within exclusive file ownership (`tsconfig.json`, `package.json`, `devvit.json`, `src/server/index.ts`, `dist/client/splash.html`, `dist/client/index.html`). `crates/emberfall/*` was not modified. TypeScript typechecking, server bundling, and strict CSP checks all pass.

## 5. Verification Method

Ran after final edits:
- `npm run typecheck` (`tsc --noEmit`) -> exit `0`
- `npm run build:server` (`vite build --config vite.server.config.ts`) -> exit `0`, produced `dist/server/index.cjs` (`3,073.06 kB`)
- `ls -lh dist/client/splash.html dist/client/index.html` -> `12K` and `18K`
- `! grep -nE 'https?://' dist/client/splash.html dist/client/index.html && echo "CSP_CHECK_PASSED"` -> `CSP_CHECK_PASSED` (exit `0`)
