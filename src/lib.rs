// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]
// use this if we can
#![cfg_attr(feature = "nightly", feature(macro_reexport))]

#[macro_use]
extern crate error_chain;
// export
#[cfg_attr(feature = "nightly", macro_reexport(json, json_internal))]
#[macro_use]
extern crate serde_json;
extern crate byteorder;

/// Error handling is assisted by the `error_chain` crate
pub mod errors;

use std::io;
use std::io::{Read, Write};
use std::panic;
use std::result::Result as StdResult;
use std::error::Error as StdError;
use serde_json::Value;
use error_chain::ChainedError;
use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};
use errors::*;

/// Writes the given JSON data to stdout, thereby 'sending' a message
/// back to Chrome. *If you are on stable, then you also need to import macros
/// from the `serde_json` crate.*
///
/// # Example
/// 
/// ```
/// #[macro_use]
/// extern crate chrome_native_messaging;
/// #[macro_use]
/// extern crate serde_json;
///
/// fn main() {
///     send!({ "msg": "Hello, world!" });
/// }
/// ```
#[macro_export]
macro_rules! send {
    ($($json:tt)+) => {
        let v = json!($($json),+);
        $crate::write_output(::std::io::stdout(), &v).unwrap();
    }
}

/// Reads input from a stream, decoded according to
/// Chrome's own documentation on native messaging.
/// (https://developer.chrome.com/extensions/nativeMessaging)
///
/// 1. A 32bit unsigned integer specifies how long the message is.
/// 2. The message is encoded in JSON 
pub fn read_input<R: Read>(mut input: R) -> Result<Value> {
    match input.read_u32::<NativeEndian>() {
        Ok(length) => {
            //println!("Found length: {}", length);
            let mut buffer = vec![0; length as usize];
            input.read_exact(&mut buffer)?;
            let value = serde_json::from_slice(&buffer)?;
            Ok(value)
        }
        Err(e) => {
            match e.kind() {
                io::ErrorKind::UnexpectedEof => bail!(ErrorKind::NoMoreInput),
                _ => Err(e.into()),
            }
        }
    }
}

/// Writes an output from a stream, encoded according to
/// Chrome's documentation on native messaging.
/// (https://developer.chrome.com/extensions/nativeMessaging)
///
/// # Example
/// 
/// ```
/// extern crate chrome_native_messaging;
/// #[macro_use]
/// extern crate serde_json;
/// use std::io;
/// 
/// fn main() {
///     let v = json!({ "msg": "Some other message" });
///     chrome_native_messaging::write_output(io::stdout(), &v)
///         .expect("failed to write to stdout");
/// }
/// ```
pub fn write_output<W: Write>(mut output: W, value: &Value) -> Result<()> {
    let msg = serde_json::to_string(value)?;
    let len = msg.len();
    // Chrome won't accept a message larger than 1MB
    if len > 1024 * 1024 {
        bail!(ErrorKind::MessageTooLarge(len))
    }
    output.write_u32::<NativeEndian>(len as u32)?;
    output.write_all(msg.as_bytes())?;
    output.flush()?;
    Ok(())
}

/// Handles a panic in the application code, by sending
/// a message back to Chrome before exiting.
fn handle_panic(info: &std::panic::PanicInfo) {
    let msg = match info.payload().downcast_ref::<&'static str>() {
        Some(s) => *s,
        None => match info.payload().downcast_ref::<String>() {
            Some(s) => &s[..],
            None => "Box<Any>",
        }
    };
    send!({
        "status": "panic",
        "payload": msg,
        "file": info.location().map(|l| l.file()),
        "line": info.location().map(|l| l.line())
    });
}

/// Starts an 'event loop' which waits for input from Chrome, and then calls
/// the callback with the message.
/// Despite its name, nothing about this function is asynchronous so if you
/// want really efficient performance while running async tasks,
/// you should probably write your own loop.
pub fn event_loop<F, E>(callback: F)
    where F: Fn(serde_json::Value) -> StdResult<(), E>,
          E: StdError
{
    panic::set_hook(Box::new(handle_panic));

    loop {
        // wait for input
        match read_input(io::stdin()) {
            Ok(v) => {
                if let Err(e) = callback(v) {
                    let text = format!("{}", e);
                    send!({
                              "error": text
                          });
                }
            }
            Err(e) => {
                // if the input stream has finished, then we exit the event loop
                if let ErrorKind::NoMoreInput = *e.kind() {
                    break;
                }
                let text = format!("{}", e.display_chain());
                send!({
                          "error": text
                      });
            }
        }
    }
}
