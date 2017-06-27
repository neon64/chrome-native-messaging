use std::io;
use serde_json;


// Create the Error, ErrorKind, ResultExt, and Result types
error_chain! {

    foreign_links {
        Io(io::Error);
        Serde(serde_json::Error);
    }

    errors {
        /// Chrome restricts message sizes to a maximum of 1MB
        MessageTooLarge(size: usize) {
            description("message too large")
            display("message too large: {} bytes", size)
        }
        NoMoreInput {
            description("EOF received")
            display("the input stream reached the end")
        }
    }

}
