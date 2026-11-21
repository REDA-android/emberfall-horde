# Explorer 2 Research Handoff: Devvit Hackathon 2026, Social/Daily Patterns & Horde-Survivor Mechanics

**Core Finding:** **EMBERFALL: DAILY HORDE** already has a lean pure-Rust Macroquad WASM core (`931 KB` `.wasm` + `37 KB` JS glue) and a working Devvit Web + Redis skeleton, but it is currently missing the **inline-splash vs. expanded-fullscreen split (`requestExpandedMode`)**, **UTC `scheduler` auto-posting**, **Devvit privacy/TTL compliance (`expire` + `onPostDelete`)**, and **async social/daily hooks** (Champion Ghost Replays, Subreddit Siege Goal, Winner-Picks-Tomorrow's-Mutator, Wordle-style Emoji Build Share Cards, Weapon Evolutions, Risk-Reward Shrines, and Fever Combo Multipliers) that separate top Devvit hackathon winners from standard arcade ports.

---

## 1. Observation

### 1.1 Current Codebase Baseline (Verified Locally)
- **Bundle Footprint (`dist/client/`):** `emberfall.wasm` is `931K`, `mq_js_bundle.js` is `37K`, `index.html` is `3.4K` (total `< 1 MB`).
- **Entrypoints & Triggers (`devvit.json:1-42`):**
  - Only a single entrypoint exists: `post.entrypoints.default` (`entry: "index.html"`, `height: "tall"`). There is no separate `splash` inline card vs `game` expanded entrypoint.
  - `triggers` only has `onAppInstall: "/internal/on-app-install"`. There is no `scheduler` cron task for daily UTC posts and no `onPostDelete` trigger.
- **Client-Server Bridge (`dist/client/index.html:34-71`):**
  - Immediately calls `load('./emberfall.wasm')` on page load (even when sitting inline in a Reddit feed).
  - Registers `emberfall_get_day_number`, `emberfall_on_run_complete` (`POST /api/submit-run`), `emberfall_share_comment` (`POST /api/share-comment`), and `emberfall_play_sfx`.
  - **Gap:** `GET /api/init` is never called by `index.html` or Rust, so the game does not receive `leaderboard`, `bossKillsToday`, `streak`, or `username` from Redis.
- **Server & Redis (`src/server/index.ts:63-244`):**
  - Uses `redis.zRange`, `redis.zAdd`, `redis.zScore`, `redis.incrBy`, `redis.get/set`, `reddit.submitCustomPost`, and `reddit.submitComment({ runAs: 'USER' })`.
  - **Gaps:** No `redis.expire(...)` TTLs on user-identifying keys; no `onPostDelete` cleanup; no rate limiting; no server-side run plausibility/session validation; no ghost replay storage; no collective subreddit goal beyond a raw `bossKillsToday` integer.

### 1.2 Reddit's Game of the Day Hackathon 2026 (Web Research)
- **Official Portal & Rules:** [redditgameofday2026.devpost.com](https://redditgameofday2026.devpost.com/) / [tierones.io summary](https://tierones.io/opportunities/reddit-daily-game-2026) / [startupgrantsindia.com](https://www.startupgrantsindia.com/reddit-game-of-the-day-hackathon-2026)
- **4 Equal Judging Criteria:**
  1. **Delightful UX** (instant clarity, responsive mobile-first controls, low friction)
  2. **Polish & Launch Readiness** (close to launch quality, strict compliance with [Devvit Rules](https://developers.reddit.com/docs/devvit_rules))
  3. **Reddit-like & Community-Minded** (plays naturally inside a subreddit thread, sparks comments/rivalries/collective effort)
  4. **Fresh Recurring Content** (compelling daily UTC loop that rewards coming back every day)
- **Prize Pool & Submission:** `$35,000` total (`$15,000` Best Daily Game, `$6,000` Best Ported Daily Game, `10 × $1,000` Honorable Mentions). Requires app link on `developers.reddit.com`, root `README.md`, playable post in a public test subreddit (`< 200` members), and optional `< 1 min` demo video. Do **not** publish for public App Review before hackathon judging concludes ([r/Devvit](https://www.reddit.com/r/Devvit/)).

### 1.3 Proven Devvit Web + Redis Patterns (GitHub & `developers.reddit.com`)
| Pattern | Reference Repo / App | Concrete Implementation Observed |
|---|---|---|
| **Inline Splash → Expanded Fullscreen** | [`Sidem/HexLife-Devvit`](https://github.com/Sidem/HexLife-Devvit), [`ult666666/blast-radius-daily`](https://github.com/ult666666/blast-radius-daily/blob/main/SUBMISSION.md), [`projectile-merge`](https://developers.reddit.com/apps/projectile-merge), [Devvit Launch Screen Docs](https://developers.reddit.com/docs/capabilities/server/launch_screen_and_entry_points/launch_screen_customization) | Define `default` (inline feed card `splash.html` showing daily mutator, top scores, boss progress, and a **Play** CTA) + `game` (`game.html` full-screen WASM). Call `requestExpandedMode(event, 'game')` synchronously inside a trusted click handler. **Crucial for WASM:** Do NOT auto-run WebGL/WASM loops in the feed card (`HexLife-Devvit`: *"Starts paused — play is explicit so large grids don't lag phones scrolling past in the feed"*). |
| **WASM + Strict CSP Compliance** | [`Sidem/HexLife-Devvit`](https://github.com/Sidem/HexLife-Devvit), [`gamelocate-irl-app`](https://developers.reddit.com/apps/gamelocate-irl-app) | Devvit Web CSP blocks external `fetch()` and external CDNs. Bundle/inline all WASM, JS, and assets in `dist/client/` (same-origin). Drop `.map` sourcemaps from publish builds. |
| **Idempotent UTC Daily Scheduler** | [`polar-latch`](https://developers.reddit.com/apps/polar-latch), [`episode-ranker`](https://developers.reddit.com/apps/episode-ranker), [`Scores`](https://developers.reddit.com/apps/scoresgame), [Devvit Scheduler Docs](https://developers.reddit.com/docs/capabilities/server/scheduler) | Configure `scheduler.tasks` in `devvit.json` (e.g. hourly or `5 0 * * *` UTC) hitting `/internal/scheduler/daily-post`. Use an atomic Redis `SET NX` lock (`pending:<id>`, 10m TTL) + `daily:<YYYYMMDD>` post-ID record so retries never duplicate posts. |
| **Privacy, TTLs & `onPostDelete`** | [`polar-latch`](https://developers.reddit.com/apps/polar-latch), [`brainsprint-2d`](https://developers.reddit.com/apps/brainsprint-2d), [Devvit Redis Docs](https://developers.reddit.com/docs/capabilities/server/redis) | Reddit requires removing deleted user IDs/names within 30 days. Set `redis.expire(key, 30 * 86400)` on daily leaderboards/meta/ghosts, and handle `onPostDelete` trigger in `devvit.json` to purge post-scoped user data immediately. Note: Devvit Redis does **not** support pipelining, plain `SET`, or `LIST` — use `ZSET`, `HASH`, `STRING`, `BITFIELD`. |
| **Leaderboard UX (Top 10 + Self Rank + Practice vs Ranked)** | [`Gem Rush`](https://developers.reddit.com/apps/gem-rush-daily), [`polar-latch`](https://developers.reddit.com/apps/polar-latch) | Show Top 10 for the UTC day; if the player is outside Top 10, display their own rank + percentile + score pinned below. Offer **Practice Mode** (or logged-out play) with a prompt to sign in to save Daily Ranked scores. |
| **Winner-Shapes-Tomorrow Meta-Hook** | [`henrygardner30/the-daily-zomble`](https://github.com/henrygardner30/the-daily-zomble) | A 1-control zombie survival Devvit game where *"the players who beat the high score get to change tomorrow's game for the whole community."* Creates massive next-day retention and comment-thread politics. |
| **Share-to-Comments Compliance & Virality** | [Devvit Rules](https://developers.reddit.com/docs/devvit_rules), [`One Shot Puzzle`](https://developers.reddit.com/apps/one-shot-puzzle), [`Scores`](https://developers.reddit.com/apps/scoresgame) | Must use `runAs: 'USER'` on explicit user button tap (already done in `src/server/index.ts:233`). Top games include a visual/emoji summary of *how* they won (build/mutator/streak) + idempotency guard so users don't accidentally spam. |
| **Server-Side Validation & Anti-Cheat** | [`polar-latch`](https://developers.reddit.com/apps/polar-latch), [`One Shot Puzzle`](https://developers.reddit.com/apps/one-shot-puzzle), [`Fault Line`](https://devpost.com/software/fault-line-daily-police-vs-thief-chase) | Never blindly trust raw client scores. Validate session start timestamp (`/api/start-run` issuing a short-lived `runId` in Redis), elapsed run duration (`>= 10s`, `<= 120s`), and plausibility bounds (max kills/sec, rate limit `rate:<userId>` 60s). |

### 1.4 90-Second Horde Survivor & Async Social Design (Game Design Research)
- **What Makes Short Survivor Runs Addictive** ([Sunstrike Roguelite Design Analysis](https://sunstrikestudios.com/en/blog/the_best_roguelite_games_to_play_in_2025/), [DRG: Survivor Review](https://lootlevelchill.com/reviews/deep-rock-galactic-survivor-review/), [Brotato Community](https://steamcommunity.com/app/1942280/discussions/0/3811786028489383380/)):
  - **Zero slow start:** Action must escalate immediately; players feel powerful quickly and hit a build "snap point" (synergy/evolution) before the boss.
  - **Legible RNG & Build Drafting:** Players want clear, meaningful synergies (e.g., *Vampire Survivors* Weapon + Passive = **Evolved Weapon**) rather than flat `+10% damage` cards.
  - **Risk-Reward Opt-In:** In a 90s fixed-timer daily seed, high-skill players need ways to **pull risk toward themselves** to separate their score on the leaderboard (e.g., *Blood Shrines* / *Cursed Obelisks* that spawn an Elite wave for a `x1.5` score surge or instant Evolution).
  - **Audio/Visual Clarity:** Crisp hit-stops, dings on XP/kill-streaks, and readable silhouettes when hordes peak.
- **Ghost Replays in Arcade/Daily Games** ([belote-cli](https://libraries.io/pypi/belote-cli), [Dash Implementor's Guide](https://www.codenameone.com/developer-guide/dash-implementors-guide/)):
  - Racing against yesterday's/today's **#1 Champion Ghost** (or the player's own PB ghost) turns a solo run into an asynchronous race without real-time networking. At 2 Hz over 90 seconds, a ghost path is only `180` `(x, y)` pairs (`~1.4 KB` JSON/binary in Redis).

---

## 2. Logic Chain

1. **Observation 1.1 & 1.3 (Inline vs Expanded):** Loading a `931 KB` WebGL/WASM binary directly in the Reddit feed (`index.html` as `default`) risks feed scroll lag on mobile and cramps 1-thumb drag controls inside a `tall` card. Splitting into a lightweight `splash.html` (showing today's Daily Mutator, Subreddit Boss Siege bar, Top 3, and **PLAY** -> `requestExpandedMode(event, 'game')`) directly satisfies **Delightful UX** and **Polish**.
2. **Observation 1.1 & 1.3 (Automation & Compliance):** Without `scheduler.tasks` in `devvit.json`, moderators must manually trigger `/internal/menu/post-create` every day; without `redis.expire` (30d) and `onPostDelete`, user IDs persist indefinitely in violation of Devvit privacy expectations. Adding both is low-effort and directly protects **Polish & Launch Readiness** and **Fresh Recurring Content**.
3. **Observation 1.1 & 1.3 (Social/Community Hooks):** Currently `/api/init` is unused by the client, and `/api/share-comment` only posts 3 text lines. Wiring `/api/init` into Rust + adding **#1 Champion Ghost Replay**, **Subreddit Collective Siege Goal**, **Winner-Picks-Tomorrow's-Mutator**, and an **Emoji Build Share Card** transforms EMBERFALL from a solo arcade game into a **subreddit-native daily ritual**, maximizing **Reddit-like & Community-Minded** judging points.
4. **Observation 1.4 (90-Second Skill Ceiling):** In a 90-second seeded run, if everyone survives to 90s and kills the boss, scores cluster unless players have **risk-reward score multipliers** (Fever Combo Meter + Blood Shrines) and **build-defining Weapon Evolutions** to optimize.

---

## 3. Caveats

- `read_url_content` could not resolve external hostnames in this sandbox (`dial tcp: no such host`), so all external web findings rely on `google_search` snippets from `developers.reddit.com`, `github.com`, `devpost.com`, and `reddit.com`.
- `crates/emberfall/src/main.rs` was not read line-by-line (only `README.md`, `devvit.json`, `src/server/index.ts`, and `dist/client/index.html` were inspected). Specific Rust weapon/card details should be verified in `crates/emberfall/src/weapons.rs` and `main.rs` before implementation.
- Reddit's `setUserFlair` API requires moderator-level app permissions and subreddit flair templates; if flair permissions complicate App Review or user consent, in-game/leaderboard **Badges + Titles** are a safer zero-permission fallback.

---

## 4. Conclusion: Prioritized Recommendations for EMBERFALL: DAILY HORDE

### P0 — Critical for Hackathon Compliance, Mobile UX & Daily Automation
1. **Inline Splash + `requestExpandedMode` Fullscreen Split (`devvit.json`, `src/client/splash.html`, `src/client/index.html`):**
   - Make `default` a crisp, zero-WASM dark-fantasy feed card (`< 30 KB` HTML/CSS/JS) fetching `/api/init` to display: **Today's Dungeon # & Mutator**, **Subreddit Dragon Slays / Siege Progress**, **Current #1 Champion & Top 3**, and a big **"⚔️ ENTER DUNGEON"** button calling `requestExpandedMode(event, 'game')`.
   - Load `emberfall.wasm` only inside the `game` entrypoint (`expanded` fullscreen on mobile = unobstructed 1-thumb virtual joystick).
2. **Automatic UTC Daily Post Scheduler + Idempotency Lock (`devvit.json` + `src/server/index.ts`):**
   - Add `scheduler.tasks` (`cron: "0 * * * *"` UTC) calling `/internal/scheduler/daily-post` with an atomic Redis `SET NX` lock (`ef:lock:<seed>`) + `ef:post:<seed>` check so each UTC day auto-posts once reliably.
3. **Devvit Privacy Compliance & Anti-Cheat (`devvit.json` + `src/server/index.ts`):**
   - Add `onPostDelete` trigger to purge post-scoped keys; add `redis.expire(..., 30 * 86400)` on daily `ef:lb:*`, `ef:meta:*`, `ef:ghost:*`.
   - Add `POST /api/start-run` returning a signed/Redis-stored `runId` + `startedAt`, and validate elapsed time (`>= 10s`, `<= 130s`) + rate limit (`rate:<userId>`) in `/api/submit-run`.
4. **Wire `/api/init` into Rust WASM:**
   - Pass `username`, `streak`, `personalBest`, `myRank`, `totalPlayers`, `bossKillsToday`, and `topGhost` into Macroquad on boot so the in-game title/HUD/post-run screen shows live Reddit community context.

### P1 — High-Impact Social & Community Differentiators ("Win the Hackathon" Hooks)
5. **#1 Champion Ghost Replay in Pure Rust (`src/server/index.ts` + `crates/emberfall/`):**
   - Record player `(x, y)` every `0.5s` (`180` samples = `~1.4 KB` compact array) during the 90s run.
   - When someone sets the #1 Daily Score, save their compact path in `ef:ghost:<sub >:<seed>`.
   - On `/api/init`, return the #1 ghost path to the client; Macroquad interpolates and renders a translucent golden/spectral **"👑 u/Champion"** phantom hero dodging alongside the player!
6. **Subreddit Collective Siege Goal (Community Boss Meter):**
   - Upgrade `bossKillsToday` into a shared **Subreddit Siege Milestone** (e.g., `25 / 50 / 100 Dragon Slays` today unlocks **+10% Ember Score Bonus** or **Golden Aura** for the whole subreddit tomorrow, displayed on both the Splash card and in-game HUD).
7. **Winner-Picks-Tomorrow's-Mutator (*The Daily Zomble* Hook):**
   - Allow the current #1 Daily Champion (or Top 3) on the post-run screen to **vote/select tomorrow's Daily Mutator** (e.g., *Blood Moon*, *Arcane Surge*, *Swarm Rush*, *Glass Cannon*), stored in `ef:mutator-vote:<sub >:<tomorrowSeed>` and announced in tomorrow's post title/splash!
8. **Rich Wordle-Style Emoji Build Share Card (`/api/share-comment`):**
   - Expand the comment template to include the player's **Streak**, **Rank**, **Daily Mutator**, and **Weapon Build Emoji Strip** (e.g., `⚔️🔥⚡🔮 · Lv.14 · 🔥 4d Streak · Top 3%`), sparking build comparisons in the comments.

### P2 — 90-Second Horde-Survivor Depth & Juice
9. **2–3 Weapon Evolutions (Synergy Snap):**
   - Pair existing weapons + passives so hitting Lv.3+ triggers a dramatic **Evolved Form** (e.g., *Ember Bolts + Arcane Surge -> Meteor Barrage*; *Runic Orbits + Dash -> Inferno Vortex*) around the `45s–55s` mark right before the Ember Lich Dragon awakens.
10. **Risk-Reward "Blood Shrines" + Fever Combo Multiplier:**
    - Spawn 1–2 deterministic **Blood Shrines** on the map each daily run: standing inside for `2.0s` spawns an Elite mini-boss + grants `+0.5x` Score Multiplier & guaranteed Evolution/Upgrade card.
    - Add a **Kill-Streak Fever Meter** (`x1.0` -> `x2.5` score/XP) that decays if you stop killing/kiting, rewarding aggressive 1-thumb positioning over passive corner-camping.

---

## 5. Verification Method

- **Codebase Claims:** Re-inspect `devvit.json`, `src/server/index.ts`, `dist/client/index.html`, and `ls -lh dist/client/`.
- **Build & Test Commands (for any future implementation):**
  - Rust unit tests: `cd crates/emberfall && cargo test`
  - Full WASM + Devvit server build: `npm run build`
  - Verify `dist/client/splash.html` + `dist/client/index.html` contain zero external URLs (CSP compliance) and total bundle stays `< 2 MB`.
  - Live Reddit verification: `npx devvit upload` -> `npx devvit playtest <test-subreddit>` -> test inline splash -> `requestExpandedMode` on mobile & desktop, `/api/init`, `/api/start-run` + `/api/submit-run` + `/api/share-comment` (as `USER`), and scheduler cron trigger.
