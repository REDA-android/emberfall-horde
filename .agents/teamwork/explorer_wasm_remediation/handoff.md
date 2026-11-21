# Handoff Report — WASM Build Remediation (`explorer_wasm_remediation`)

**Summary**: `wasm32-unknown-unknown` IS already installed on disk in `/home/antigravity/.rustup`, but the container environment overrides `RUSTUP_HOME=/usr/local/rustup` (a read-only mount lacking `wasm32-unknown-unknown`). Running the build with `RUSTUP_HOME=/home/antigravity/.rustup` (e.g., `RUSTUP_HOME=/home/antigravity/.rustup npm run build`) resolves `std` and compiles `wasm32-unknown-unknown` cleanly in this sandbox.

---

## 1. Observation

1. **Environment variables (`env | grep -E 'RUST|CARGO|PATH|HOME|TMP'`)**:
   - `HOME=/home/antigravity`
   - `RUSTUP_HOME=/usr/local/rustup`
   - `CARGO_HOME` is unset (defaults to `/home/antigravity/.cargo`)
   - `which rustc cargo rustup` -> `/usr/local/bin/rustc`, `/usr/local/bin/cargo`, `/usr/local/bin/rustup`

2. **Filesystem mounts (`mount | head -n 40`)**:
   - `/` is mounted `tmpfs (ro,relatime,...)` (read-only), and `/usr` overlays are read-only, making `/usr/local/rustup` read-only (`Read-only file system (os error 30)` on `rustup target add`).
   - `/dev/vdb on /home/antigravity type xfs (rw,noatime,...)` is read-write.

3. **Default `rustup show` (with `RUSTUP_HOME=/usr/local/rustup`)**:
   ```
   rustup home:  /usr/local/rustup
   installed targets:
     x86_64-unknown-linux-gnu
   ```
   `ls -la /usr/local/rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/` confirms only `x86_64-unknown-linux-gnu` is present (dated `Nov 12 19:08`).

4. **Filesystem search (`find / -name "*wasm32-unknown-unknown*" 2>/dev/null`)**:
   - `/home/antigravity/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/wasm32-unknown-unknown`
   - `/home/antigravity/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/manifest-rust-std-wasm32-unknown-unknown`
   - `/home/antigravity/REDA-android/emberfall-horde/crates/emberfall/target/wasm32-unknown-unknown`

5. **Overridden `RUSTUP_HOME=/home/antigravity/.rustup rustup show`**:
   ```
   rustup home:  /home/antigravity/.rustup
   installed targets:
     wasm32-unknown-unknown
     x86_64-unknown-linux-gnu
   ```
   Both `/usr/local/rustup` and `/home/antigravity/.rustup` have `rustc 1.100.0 (0ef1bc386 2026-11-09)`.

6. **Cargo cache & build script**:
   - `/home/antigravity/.cargo` contains `registry/` (cached crates from `Nov 21 14:50`).
   - `/home/antigravity/REDA-android/emberfall-horde/package.json:7-9`:
     - `"build:wasm": "cd crates/emberfall && cargo build --release --target wasm32-unknown-unknown && cp target/wasm32-unknown-unknown/release/emberfall.wasm ../../dist/client/emberfall.wasm"`
     - `"build": "npm run build:wasm && npm run build:server"`
   - Current `dist/client/emberfall.wasm` and `crates/emberfall/target/wasm32-unknown-unknown/release/emberfall.wasm` are both `952858` bytes (`Nov 21 14:52`).

7. **Read-only verification (`RUSTUP_HOME=/home/antigravity/.rustup cargo check --release --target wasm32-unknown-unknown` in `crates/emberfall`)**:
   - Exited with code `0` in `4.64s`:
     `Finished release profile [optimized] target(s) in 4.64s`

---

## 2. Logic Chain

1. From Observation 1 & 2, `RUSTUP_HOME=/usr/local/rustup` is explicitly exported in the container environment and points to a read-only filesystem without `wasm32-unknown-unknown` (Observation 3). That explains both `error[E0463]: can't find crate for std` during `npm run build:wasm` and `Read-only file system (os error 30)` when M1 Worker ran `rustup target add wasm32-unknown-unknown`.
2. From Observation 4 & 5, a complete, writable rustup installation already exists at `/home/antigravity/.rustup` with `stable-x86_64-unknown-linux-gnu` (`rustc 1.100.0`) and `wasm32-unknown-unknown` already installed.
3. From Observation 6 & 7, `/home/antigravity/.cargo` already has the dependency registry cached, and overriding `RUSTUP_HOME=/home/antigravity/.rustup` allows `cargo` to compile `crates/emberfall` for `wasm32-unknown-unknown` in 4.64s without needing network access or modifying system mounts.
4. Therefore, a Worker can immediately rebuild `dist/client/emberfall.wasm` in this sandbox by setting `RUSTUP_HOME=/home/antigravity/.rustup` when invoking `npm run build:wasm` / `npm run build`.

---

## 3. Caveats

- `cargo check --release --target wasm32-unknown-unknown` verified compilation and `std` resolution; `cargo build --release --target wasm32-unknown-unknown` (full linking to `.wasm`) was not executed in order to keep this investigation read-only with respect to `dist/client/emberfall.wasm`.
- Any future Worker or Auditor running `npm run build` / `npm run build:wasm` in a fresh shell in this container must explicitly prefix `RUSTUP_HOME=/home/antigravity/.rustup` (unless `package.json` or the shell exports it), because the container's default environment sets `RUSTUP_HOME=/usr/local/rustup`.

---

## 4. Conclusion

**Remediation is immediately possible in this sandbox — no host intervention is required.**

Exact commands for the remediation Worker:
```bash
cd /home/antigravity/REDA-android/emberfall-horde
RUSTUP_HOME=/home/antigravity/.rustup npm run build
```
(Or specifically `RUSTUP_HOME=/home/antigravity/.rustup npm run build:wasm` followed by `npm run build:server`.)

Then verify that `dist/client/emberfall.wasm` has an updated timestamp and matches `crates/emberfall/target/wasm32-unknown-unknown/release/emberfall.wasm`, and re-run the Forensic Auditor with `RUSTUP_HOME=/home/antigravity/.rustup npm run build`.

---

## 5. Verification Method

1. **Check toolchain availability**:
   ```bash
   RUSTUP_HOME=/home/antigravity/.rustup rustup target list --installed
   ```
   Expected: includes `wasm32-unknown-unknown` and `x86_64-unknown-linux-gnu`.

2. **Check WASM compilation (read-only wrt `dist/`)**:
   ```bash
   cd /home/antigravity/REDA-android/emberfall-horde/crates/emberfall
   RUSTUP_HOME=/home/antigravity/.rustup cargo check --release --target wasm32-unknown-unknown
   ```
   Expected: exit code `0`.

3. **Post-remediation verification (after Worker runs `RUSTUP_HOME=/home/antigravity/.rustup npm run build`)**:
   ```bash
   ls -la /home/antigravity/REDA-android/emberfall-horde/dist/client/emberfall.wasm \
          /home/antigravity/REDA-android/emberfall-horde/crates/emberfall/target/wasm32-unknown-unknown/release/emberfall.wasm
   sha256sum /home/antigravity/REDA-android/emberfall-horde/dist/client/emberfall.wasm \
             /home/antigravity/REDA-android/emberfall-horde/crates/emberfall/target/wasm32-unknown-unknown/release/emberfall.wasm
   ```
   Expected: identical SHA-256 hashes and updated mtime after the latest Rust source edits.
