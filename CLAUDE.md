# Project Instructions

chatclient-plugin-sicompass was split out of the
[sicompass](https://github.com/friendlyflow/sicompass) workspace, and its git
history before that point is the history of `lib/lib_chatclient` (earlier
`lib/lib_chatclient-rs`) there. Work on it is usually driven from a sicompass
checkout next to this one (`../sicompass`), whose `/commit-and-push`,
`/release`, `/sync` and `/update-cargo` take this repo's name as their first
argument and then follow the skills in this repo's `.claude/skills/`.

It is a sicompass **plugin process**: a program (`src/main.rs`) built with the
SDK's `plugin` feature, which sicompass starts and talks to over its stdin and
stdout. It runs with the user's rights. The Store installs it from this repo's
GitHub releases, one build per platform. The plugin platform is described in
`../sicompass/docs/plugin-platform.md`.

- `plugin.json` is the manifest. Its `name` is `chatclient` and its
  `displayName` `chat client` is the settings section (the keys the built-in
  had, so saved values carry over). Permissions, which declare what the plugin
  does and are shown to the user before install: `"allowedHosts": ["*"]` (the
  homeserver is the user's choice) and `storage`.
- `locales/<lang>.ftl`, every id prefixed `chatclient-`, in all four
  languages. `src/localize.rs` asks the app (`host::translate`), and in the
  unit tests, which run outside sicompass, reads `en-US.ftl`.
- `src/lib.rs` is the provider (`ChatClientProvider`, `impl Plugin`), and
  `src/main.rs` makes it the program.

## How it works

- **HTTP** goes through `src/http.rs`, a small client in the shape of
  `reqwest::blocking` over `ureq` (rustls with ring and bundled roots, so the
  static musl build needs no system TLS library). The tests run it against a
  wiremock homeserver. Never against a real one.
- **Every call from the app has a 10-second deadline**, after which the app
  ends the plugin. Requests made on a call (sign-in, sending, loading earlier
  messages) time out after 8 seconds (`UI_REQUEST_TIMEOUT`), so a slow
  homeserver is an error the user sees.
- **`/sync`** long-polls on a thread of its own (`sync::SyncController`). It
  merges each response into the shared cache, saves the sync position, and
  raises the flag that `poll` turns into `needs_refresh`. After a failure it
  waits 10 seconds before reconnecting. `cleanup` stops it.
- **The sign-in** (access token, user id, sync position) is kept in the
  plugin's storage folder (`sicompass_sdk::plugin::storage_dir()/chat.json`,
  in the shape of a settings file), since a plugin cannot write the app's
  settings. The settings the manifest declares are read at `init`, and a saved
  sign-in takes over from them. Every write to that file holds `file_lock`,
  which the sync thread shares.
- **Undo** entries are `ProviderOp`s: the action's name and its fields as an
  FFON list (`encode_op`, `decode_op`).
- Registration's browser fallback opens through
  `sicompass_sdk::plugin::desktop::open_url`.

## Environment (Nix)

The toolchain comes from the flake dev shell in [flake.nix](flake.nix): Rust
from rust-overlay with this computer's plugin target (static musl on Linux,
which nixpkgs' rustc has no std for) and `jq`. Nothing is installed
system-wide.

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
  `cargo test`, and `./scripts/release-plugin.sh --dry-run`, which also builds
  this computer's release and verifies it the way the Store will.
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

The SDK comes from crates.io (the source is `../sicompass-plugin-sdk`). The
commented-out `[patch]` in `Cargo.toml` is for working on them together, and
stays commented on main.

A release has one archive per platform. The release workflow builds them on
five runners (Linux x86_64 and arm64 as static musl, macOS arm64 and x86_64,
Windows x86_64), then packs, signs and verifies them in one job.
