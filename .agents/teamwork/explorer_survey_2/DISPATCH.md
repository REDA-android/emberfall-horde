# DISPATCH

## 2026-11-21T15:11:01Z
Sender: 5ccb6c664a1506539f23537429815fc5
Priority: MESSAGE_PRIORITY_HIGH

You are Explorer 2 (Devvit Hackathon & Social/Daily Game Researcher) for EMBERFALL: DAILY HORDE.

MANDATORY FIRST STEP: Read `/home/antigravity/REDA-android/emberfall-horde/.agents/teamwork/ORIGINAL_REQUEST.md` verbatim before anything else.

Your state folder is: `/home/antigravity/REDA-android/emberfall-horde/.agents/teamwork/explorer_survey_2`
Project root: `/home/antigravity/REDA-android/emberfall-horde`

Objective:
Conduct comprehensive research across Google, GitHub, GitLab, Reddit Devvit docs/examples, and itch.io/game dev blogs on:
1. **Reddit's Game of the Day Hackathon 2026 / Devvit Web Games**: judging criteria, winning patterns, what makes a Devvit Web game stand out (retention, community/subreddit engagement, daily replayability, mobile-first 1-thumb UX, fast load time / small WASM size, CSP compliance).
2. **Devvit Web + Redis Social & Daily Hooks**: concrete patterns used by top Devvit Web games on GitHub/GitLab/Reddit:
   - Daily seeded runs (UTC midnight rollover, deterministic RNG, daily modifiers/mutators like Vampire Survivors / Slay the Spire / Balatro daily challenges)
   - Redis-backed leaderboards (global daily, subreddit-specific, percentile rank, streak tracking, personal bests, karma/flair rewards)
   - Asynchronous multiplayer / social features: **Ghost replays** (recording lightweight input/position frames in Redis and rendering yesterday's #1 or rival ghosts in-game!), **Community Boss / Global Horde HP** (subreddit-wide collective goal), **Share-to-Comments / Custom Post** (formatted Markdown/ASCII/rich score card posted as a Reddit comment or challenge link with seed/build), **Rivalries / Challenge a Redditor**
   - Devvit Web splash screen / inline post preview / mobile viewport & touch handling best practices.
3. **Vampire Survivors / Horde-Survivor Daily Game Mechanics**: what makes 2-3 minute daily survivor runs ultra-addictive (meta-builds, synergies/evolutions, risk-reward shrines/events, fever/combo multipliers, boss waves, daily mutators).

Output:
Write a well-structured, concrete, citation-backed research report to `/home/antigravity/REDA-android/emberfall-horde/.agents/teamwork/explorer_survey_2/handoff.md` with URLs/repos/patterns and prioritized recommendations tailored to EMBERFALL: DAILY HORDE's constraints (100% pure Rust Macroquad WASM + Devvit Web + Redis + strict CSP + 1-thumb mobile).
Do NOT modify any source code files. Send a concise completion message back to your caller with the handoff path.
