# chatclient-plugin-sicompass

*Matrix chat, in Sicompass.*

This plugin is part of [Sicompass](https://github.com/friendlyflow/sicompass), a
keyboard-first, accessibility-first way to use your entire computer.

The chat client shows your Matrix rooms as a list. Right enters a room, its
messages are rows, and i on the last row writes a message. Members, invites,
public rooms and spaces are lists too, and leaving a room, joining one, kicking
or banning someone can be undone with ctrl+z.

Sign in on the first screen, or register an account, on the homeserver you set
in Settings, under chat client (matrix.org by default). New messages arrive in
the background.

The chat client asks to reach any server on the internet, because your
homeserver is yours to choose. The Store shows that before you install it, and
installing it is your approval.

## Install

In Sicompass, open store, then programs, and press Enter on install next to
chatclient. The Store checks the release's signature before installing it, and
keeps it up to date.

## Building from source

```bash
nix develop          # the toolchain
cargo test           # against a mock homeserver
cargo build --release
cp target/release/chatclient-plugin plugin
```

To install a build of your own, copy `plugin.json`, the built `plugin` program
(`plugin.exe` on Windows) and `locales/` into a folder named `chatclient` in
the Sicompass plugins folder (`~/.config/sicompass/plugins/` on Linux,
`~/Library/Application Support/sicompass/plugins/` on macOS) and restart
Sicompass.

`./scripts/release-plugin.sh --dry-run` builds this computer's release, packs
it, and signs and verifies it with a throwaway key, the way a release is made.

## Related repositories

- [sicompass](https://github.com/friendlyflow/sicompass), the application
- [sicompass-plugin-sdk](https://github.com/friendlyflow/sicompass-plugin-sdk),
  the SDK, the plugin kit and the cloud backup library

## Community

Join the conversation on
[Discord](https://discord.com/channels/1464152138753249313/1464152139231137894).

## License

#### Open source license

If you are creating an open source application under a license compatible with
the GNU GPL license v3, you may use this project under the terms of the GPLv3.
See [LICENSE](LICENSE).

## Contributing

Contributions are welcome. Whether it is code, documentation, or feedback, your
input helps make computing more accessible for everyone.
