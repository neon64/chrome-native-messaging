mod errors;

pub use crate::errors::Error;
use serde::Serialize;
use serde_json::{json, Value};
use std::fmt::Display;
use std::io;
use std::io::{Read, Write};
use std::panic;

/// Writes the given JSON data to stdout, thereby 'sending' a message
/// back to Chrome. *If you are on stable, then you also need to import macros
/// from the `serde_json` crate.*
///
/// # Example
///
/// ```
/// use chrome_native_messaging::send;
/// use serde_json::json;
///
/// send!({ "msg": "Hello, world!" });
/// ```
#[macro_export]
macro_rules! send {
    ($($json:tt)+) => {{
        let v = json!($($json),+);
        $crate::send_message(::std::io::stdout(), &v)
    }}
}

/// Reads input from a stream, decoded according to
/// Chrome's own documentation on native messaging.
/// (https://developer.chrome.com/extensions/nativeMessaging)
///
/// 1. A 32bit unsigned integer specifies how long the message is.
/// 2. The message is encoded in JSON
///
/// # Example
///
/// ```
/// use std::io;
/// use chrome_native_messaging::{read_input, Error};
///
/// read_input(io::stdin())
///     .err().expect("doctest should return unexpected eof");
///
pub fn read_input<R: Read>(mut input: R) -> Result<Value, Error> {
    let mut buf = [0; 4];
    match input.read_exact(&mut buf).map(|()| u32::from_ne_bytes(buf)) {
        Ok(length) => {
            //println!("Found length: {}", length);
            let mut buffer = vec![0; length as usize];
            input.read_exact(&mut buffer)?;
            let value = serde_json::from_slice(&buffer)?;
            Ok(value)
        }
        Err(e) => match e.kind() {
            io::ErrorKind::UnexpectedEof => Err(Error::NoMoreInput),
            _ => Err(e.into()),
        },
    }
}

/// Writes an output to a stream, encoded according to
/// Chrome's documentation on native messaging.
/// (https://developer.chrome.com/extensions/nativeMessaging)
/// Takes a custom value which implements serde::Serialize.
///
/// # Example
///
/// ```
/// use chrome_native_messaging::send_message;
/// use std::io;
/// use serde::Serialize;
/// use serde_json::json;
///
/// #[derive(Serialize)]
/// struct BasicMessage<'a> {
///     payload: &'a str
/// }
///
/// send_message(io::stdout(), &BasicMessage { payload: "Hello, World! "})
///     .expect("failed to send to stdout");
/// ```
pub fn send_message<W: Write, T: Serialize>(mut output: W, value: &T) -> Result<(), Error> {
    let msg = serde_json::to_string(value)?;
    let len = msg.len();
    // Chrome won't accept a message larger than 1MB
    if len > 1024 * 1024 {
        return Err(Error::MessageTooLarge { size: len });
    }
    let len = len as u32; // Cast is safe due to size check above
    let len_bytes = len.to_ne_bytes();
    output.write_all(&len_bytes)?;
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
        },
    };
    // Ignore error if send fails, we don't want to panic inside the panic handler
    let _ = send!({
        "status": "panic",
        "payload": msg,
        "file": info.location().map(|l| l.file()),
        "line": info.location().map(|l| l.line())
    });
}

/// Starts an 'event loop' which listens and writes to
/// stdin and stdout respectively.
///
/// Despite its name implying an asynchronous nature,
/// this function blocks waiting for input.
///
/// # Example
///
/// ```
/// use chrome_native_messaging::event_loop;
/// use std::io;
/// use serde::Serialize;
/// use serde_json::{json, Value};
///
/// #[derive(Serialize)]
/// struct BasicMessage<'a> {
///     payload: &'a str
/// }
///
/// event_loop(|value| match value {
///     Value::Null => Err("null payload"),
///     _ => Ok(BasicMessage { payload: "Hello, World!" })
/// });
///
/// ```
pub fn event_loop<T, E, F>(callback: F)
where
    F: Fn(serde_json::Value) -> Result<T, E>,
    T: Serialize,
    E: Display,
{
    panic::set_hook(Box::new(handle_panic));

    loop {
        // wait for input
        match read_input(io::stdin()) {
            Ok(v) => match callback(v) {
                Ok(response) => send_message(io::stdout(), &response).unwrap(),
                Err(e) => send!({ "error": format!("{}", e) }).unwrap(),
            },
            Err(e) => {
                // if the input stream has finished, then we exit the event loop
                if let Error::NoMoreInput = e {
                    break;
                }
                send!({ "error": format!("{}", e) }).unwrap();
            }
        }
    }
}
