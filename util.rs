
use std::num::strconv::{from_str_bytes_common, ExpNone};
use std::io::stdio::{stdin};
use std::io::io_error;

/**
 * Reads two characters from stdin, which we interpret as an 8-bit hex number
 */
pub fn read_hex_char() -> Option<u8>
{
  let mut read_stream = stdin();
  let mut read_buf: ~[u8] = ~[0, 0];

  match read_stream.read (read_buf) {
    Some(_) => { from_str_bytes_common (read_buf, 16, false, false, false, ExpNone, false, false) }
    None => None
  }
}

/**
 * Reads an entire array of hex values from stdin
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
 * Converts a bitstring to a hexadecimal string for user output
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

