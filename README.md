# Chrome Native Messaging in Rust

Google Chrome allows native applications to interface with browser plugins as described in their [documentation](https://developer.chrome.com/extensions/nativeMessaging). This Rust crate provides simple functions for encoding/decoding JSON messages and handling errors during this process.

[**API Documentation**](https://docs.rs/chrome_native_messaging)

## Contributing

- It would be great to integrate this with Tokio in order to make a fully asyncronous event loop. I haven't needed it personally so far, therefore it hasn't been worth my time to implement it.
- Currently there are no integration tests with Chrome itself, to make sure that the protocol is implemented correctly. I wasn't sure how to achieve this easily, but if anybody has a suggestion then it would be welcomed.
- All contributions/suggestions welcome!!