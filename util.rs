
use std::num::strconv::{from_str_bytes_common, ExpNone};
use std::io::stdio::{stdin};
use std::io::io_error;

/**
 * stdin에서 두 문자를 읽어 8비트 16진수로 해석한다
 */
pub fn read_hex_char() -> Option<u8>
{
  let mut read_stream = stdin();
  let mut read_buf: ~[u8] = ~[0];

  if read_stream.read (read_buf).is_none() {
    return None;
  }
  let digit_1 = from_str_bytes_common (read_buf, 16, false, false, false, ExpNone, false, false);
  if digit_1.is_none() {
    return None;
  }
  if read_stream.read (read_buf).is_none() {
    return None;
  }
  let digit_2 = from_str_bytes_common (read_buf, 16, false, false, false, ExpNone, false, false);
  if digit_2.is_none() {
    return None;
  }

  Some(16 * digit_1.unwrap() + digit_2.unwrap())
}

/**
 * stdin에서 16진수 값 배열 전체를 읽는다
 */
pub fn read_hex() -> ~[u8]
{
  let mut rv: ~[u8] = ~[];
  io_error::cond.trap(|_| ()).inside(|| {
    loop {
      match read_hex_char() {
        None => { break }
        Some(hex) => {
          rv.push (hex);
        }
      }
    }
  });
  rv
}

/**
 * 사용자 출력용으로 비트열을 16진수 문자열로 변환한다
 */
pub fn u8_to_hex_string(data: &[u8]) -> ~str {
  let hex_chars = "0123456789abcdef";
  let mut rv = ~"";

  for c in data.iter() {
    rv.push_char (hex_chars[c >> 4] as char);
    rv.push_char (hex_chars[c % 16] as char);
  }
  rv
}

