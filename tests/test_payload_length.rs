use chrome_native_messaging::{send_message, Error};
use serde::Serialize;
use serde_json::json;
use std::io::sink;

#[derive(Serialize)]
struct MoreInfo {
    a: i32,
    b: i32,
}

#[test]
fn test_payload_length() {
    let m = MoreInfo { a: 0, b: 5 };

    // this is tiny it will work
    let small_res = json!({
        "will_i_work": true,
        "one_item": m
    });

    assert!(send_message(sink(), &small_res).is_ok());

    // this is almost 1024*1024 bytes long, but it should still work
    let list = " ".repeat(1024 * 1024 - 20);
    let large_res = json!({ "big_list": list });

    assert!(send_message(sink(), &large_res).is_ok());

    // this is almost 1024*1024 bytes long, but it should still work
    let list = " ".repeat(1024 * 1024 + 20);
    let too_large_res = json!({ "big_list": list });

    match send_message(sink(), &too_large_res)
        .err()
        .expect("expected error")
    {
        Error::MessageTooLarge { size: _ } => {}
        _ => panic!("expected `MessageTooLarge` error"),
    }
}
