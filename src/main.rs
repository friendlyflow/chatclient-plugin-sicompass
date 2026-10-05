//! The program sicompass starts: the chat client, served over stdin and stdout.

sicompass_sdk::plugin::main!(chatclient_plugin::ChatClientProvider);
