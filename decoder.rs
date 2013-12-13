
pub enum TokenType {
  Unsigned32,
  Unsigned64,
  VarInt,
  Bytestring(u64)
}

pub enum Token {
  Integer(u64),
  String(~[u8]),
  Invalid
}

fn decode_integer (iter: &mut Iterator<&u8>, width: int) -> Token {
  let mut success = true;
  let mut rv: u64 = 0;

  for i in range (0, width) {
    match iter.next() {
      None => { success = false; break }
      Some(ch) => { rv += (*ch as u64) << 8 * (width - i); }
    }
  }
  if success { Integer (rv) }
  else       { Invalid }
}


pub fn decode_token (iter: &mut Iterator<&u8>, expected_token: TokenType) -> Token {
  match expected_token {
    /* Fixed-width integers */
    Unsigned32 => { decode_integer (iter, 4) }
    Unsigned64 => { decode_integer (iter, 8) }
    /* Variable-width integers */
    VarInt => {
      match iter.next() {
        Some(ch) => {
          match *ch {
            0xff => decode_integer (iter, 8),
            0xfe => decode_integer (iter, 4),
            0xfd => decode_integer (iter, 2),
            n => { Integer(n as u64) }
          }
        }
        None => Invalid
      }
    }
    /* Strings */
    Bytestring(len) => {
      let mut success = true;
      let mut rv: ~[u8] = ~[];
      for _ in range (0, len) {
        match iter.next() {
          None => { success = false; break }
          Some(ch) => { rv.push (*ch); }
        }
      }
      if success { String (rv) }
      else       { Invalid }
    }
  }
}


