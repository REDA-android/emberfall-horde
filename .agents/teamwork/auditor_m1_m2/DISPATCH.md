## 2026-11-21T15:21:05Z

You are the Forensic Integrity Auditor (`teamwork_preview_auditor`) for EMBERFALL: DAILY HORDE (intermediate whole-scope audit of M1 + M2).

MANDATORY FIRST STEP: Read `/home/antigravity/REDA-android/emberfall-horde/.agents/teamwork/ORIGINAL_REQUEST.md` verbatim before anything else.
Then read `/home/antigravity/REDA-android/emberfall-horde/PROJECT.md`.

Your state folder: `/home/antigravity/REDA-android/emberfall-horde/.agents/teamwork/auditor_m1_m2`
Project root: `/home/antigravity/REDA-android/emberfall-horde`

Audit Scope (Intermediate Milestone Audit):
- `src/server/index.ts`, `devvit.json`, `tsconfig.json`, `package.json`, `dist/client/splash.html`, `dist/client/index.html`
- `crates/emberfall/src/shaders.rs`, `crates/emberfall/src/rng.rs`, `crates/emberfall/src/weapons.rs`, `crates/emberfall/src/main.rs`

Rules:
- MUST NOT modify the working tree or put source, probes/tests, or data in `.agents/teamwork/`.
- Run target-scoped builds/tests + anti-cheating checks (static analysis of `git diff`, stub/dummy/hardcoded-output detection, authentic logic verification for evolutions, shrines, ghost replay, anti-cheat, scheduler lock, and shaders).

Write your forensic audit verdict (`CLEAN` or `INTEGRITY VIOLATION` with full evidence) to `/home/antigravity/REDA-android/emberfall-horde/.agents/teamwork/auditor_m1_m2/handoff.md`, and message the caller.
