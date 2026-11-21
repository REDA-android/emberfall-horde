## 2026-11-21T15:28:52Z
You are the WASM Build Worker (`worker_wasm_build`) for EMBERFALL: DAILY HORDE.

MANDATORY FIRST STEP: Read `/home/antigravity/REDA-android/emberfall-horde/.agents/teamwork/ORIGINAL_REQUEST.md` verbatim before anything else.
Then read `/home/antigravity/REDA-android/emberfall-horde/.agents/teamwork/explorer_wasm_remediation/handoff.md`.

Your state folder: `/home/antigravity/REDA-android/emberfall-horde/.agents/teamwork/worker_wasm_build`
Project root: `/home/antigravity/REDA-android/emberfall-horde`

MANDATORY INTEGRITY WARNING:
> DO NOT CHEAT. All implementations must be genuine. DO NOT
> hardcode test results, create dummy/facade implementations, or
> circumvent the intended task. A teamwork_preview_auditor will independently
> verify your work. Integrity violations WILL be detected and your
> work WILL be rejected.

Tasks:
1. Run `RUSTUP_HOME=/home/antigravity/.rustup npm run build` in `/home/antigravity/REDA-android/emberfall-horde` to compile `crates/emberfall` to `wasm32-unknown-unknown` (`dist/client/emberfall.wasm`) and bundle `dist/server/index.cjs`.
2. If `wasm-opt` is installed (`which wasm-opt`), run `wasm-opt -Oz --all-features dist/client/emberfall.wasm -o dist/client/emberfall.wasm`.
3. If `RUSTUP_HOME=/home/antigravity/.rustup npm run build` exposes any `wasm32`-only compiler error or warning in `crates/emberfall/src/`, fix it cleanly so `RUSTUP_HOME=/home/antigravity/.rustup cargo check --manifest-path crates/emberfall/Cargo.toml --target wasm32-unknown-unknown` has 0 errors and 0 warnings.
4. Run full verification:
   - `RUSTUP_HOME=/home/antigravity/.rustup npm run build`
   - `RUSTUP_HOME=/home/antigravity/.rustup cargo test --manifest-path crates/emberfall/Cargo.toml`
   - `npm run typecheck`
   - `ls -lh dist/client/emberfall.wasm dist/client/index.html dist/client/splash.html dist/server/index.cjs`

Write your exact command outputs and updated `dist/client/emberfall.wasm` size/timestamp to `/home/antigravity/REDA-android/emberfall-horde/.agents/teamwork/worker_wasm_build/handoff.md`, and message the caller when done.
