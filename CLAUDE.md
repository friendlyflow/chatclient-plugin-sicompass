# Project Instructions

chatclient_plugin_sicompass was split out of the
[sicompass](https://github.com/friendlyflow/sicompass) workspace, and its git
history before that point is the history of `lib/lib_chatclient` (earlier
`lib/lib_chatclient-rs`) there. Work on it is usually driven from a sicompass
checkout next to this one (`../sicompass`), whose `/commit-and-push`,
`/release`, `/sync` and `/update-cargo` take this repo's name as their first
argument and then follow the skills in this repo's `.claude/skills/`.

It is a sicompass **WASM plugin**: a `cdylib` built for `wasm32-wasip2` with
`sicompass-pdk`, installed by the sicompass Store from this repo's GitHub
releases. The plugin platform is described in
`../sicompass/docs/plugin-platform.md` and `../sicompass/docs/wasm-plugins.md`.

- `plugin.json` is the manifest. Its `name` is `chatclient` and its
  `displayName` `chat client` is the settings section (the keys the built-in
  had, so saved values carry over). It asks for `"allowedHosts": ["*"]` (the
  homeserver is the user's choice; the host never lets it reach the local
  network), approved at install, and `storage`.
- `locales/<lang>.ftl`, every id prefixed `chatclient-`, in all four
  languages.

## The sandbox, and what it changes

- **HTTP** goes through `src/http.rs`, a small client in the shape of
  `reqwest::blocking`: the host's `net.fetch` in the sandbox, reqwest natively
  so the tests run against a wiremock homeserver. The host allows GET, HEAD,
  POST, PUT and DELETE (Matrix sends messages with PUT).
- **`/sync`** is a host task (`sync::SYNC_TASK`, a second instance of the
  plugin) that long-polls and passes each response on with `tasks.emit`. The
  UI instance merges it into the cache and asks for a refresh. A task's
  requests may wait two minutes, so the 30-second long poll fits. Natively the
  same loop is a thread.
- **The sign-in** (access token, user id, sync position) is kept in the
  plugin's storage folder (`/storage/chat.json`, in the shape of a settings
  file), since a plugin cannot write the app's settings. The settings the
  manifest declares are read at `init`, and a saved sign-in takes over from
  them. Only the UI instance writes that file.
- **Undo** entries are `ProviderOp`s: the action's name and its fields as an
  FFON list (`encode_op`, `decode_op`).
- Registration's browser fallback opens through `desktop.open-url`.

## Environment (Nix)

The toolchain comes from the flake dev shell in [flake.nix](flake.nix): Rust
from rust-overlay with the `wasm32-wasip2` target (nixpkgs' rustc has no `std`
for it), `wasm-tools` and `jq`. Nothing is installed system-wide.

- **Check once per session**, then stick with the answer: `command -v cargo`.
  - Non-empty: the shell is inside `nix develop`, so run `cargo ...` directly.
  - Empty: prefix every toolchain command with `nix develop -c`.
- `nix develop -c <cmd>` prints a `warning: Git tree ... is dirty` line on
  stderr first. That warning is noise, not a failure.
- Evaluate the flake through `git+file://$PWD`, never a plain path (a plain path
  copies `target/` into the store and hangs), and always under `timeout`.
- The version lives in `plugin.json` and in `[package] version` in `Cargo.toml`.
  Bump both together.

## Generated files that are committed

- `THIRD-PARTY-LICENSES.html`: `cargo about generate about.hbs -o
  THIRD-PARTY-LICENSES.html` (cargo-about 0.9.2, the version the `licenses.yml`
  workflow pins). Regenerate and commit it with any dependency change. The
  workflow fails if it drifts.

## Code Style

Follow standard Rust idioms. Use `#[allow(...)]` sparingly and only when
justified. In `README.md`, do not use em dashes or semicolons. Use commas
instead, or split into separate sentences.

## Testing

- After implementing changes, always run the tests before finishing:
  `cargo test` (natively), and `./scripts/release-plugin.sh --dry-run`, which
  also builds the component and audits its imports.
- When adding new code, write or update tests.
- If tests fail, fix the code. Never leave a task with failing tests.

## Test Integrity

- Never remove or weaken test assertions to make a failing test pass. Fix the
  code instead.
- If a test itself is genuinely wrong and needs changing, **ask the user
  first** before modifying it.

## Releasing

A release is a `vX.Y.Z` tag on `main`, equal to `plugin.json`'s version. See
`.claude/skills/release/SKILL.md`. Before tagging, run
`nix develop -c ./scripts/release-plugin.sh --dry-run` (needs the
`sicompass-plugin` tool: `cargo install --git
https://github.com/friendlyflow/sicompass-plugin-sdk sicompass-plugin`). The
release workflow signs with the `PLUGIN_SIGNING_KEY` secret and checks it
against the `PLUGIN_PUBLIC_KEY` variable, the key the sicompass store list
names. The secret key file is `~/.config/sicompass/plugin-keys/chatclient.key`
on the maintainer's machine. Never print, copy or commit it.

The SDK and the pdk (in `../sicompass-plugin-sdk`) come by git at one rev
until they are on crates.io. The commented-out
`[patch]` in `Cargo.toml` is for working on them together, and stays commented
on main.
