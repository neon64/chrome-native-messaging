# Chrome Native Messaging in Rust

Google Chrome allows native applications to interface with browser plugins as described in their [documentation](https://developer.chrome.com/extensions/nativeMessaging). This Rust crate provides simple functions for encoding/decoding JSON messages and handling errors during this process.

[**Documentation**](https://docs.rs/chrome-native-messaging)

## Contributing

- It would be great to integrate this with Tokio in order to make a fully asyncronous event loop. I haven't needed it personally so far, therefore it hasn't been worth my time to implement it.
- Currently there are *0 tests*. Yes, I know, that is a horrible abomination against the TDD gods. However its a really simple library, and I haven't thought of a good way to actually ensure that the communication protocol is implemented correctly, short of actually using it with a Chrome plugin.
- All contributions/suggestions welcome!!