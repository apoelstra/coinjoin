
use std::libc::size_t;
use std::vec::from_buf;
use std::vec::raw::to_ptr;


#[link_args = "sha-wrapper.o -lcrypto"]
extern {
  fn csha256_sum (input: *u8, len: size_t) -> *u8;
  fn csha256_destroy (input: *u8);
}

/* HASH FUNCTIONS */

/**
 * Compute a SHA256 sum of a raw bitstring
 */
pub fn sha256_sum (input: &[u8]) -> ~[u8]
{
  unsafe {
    let raw_ptr = csha256_sum (to_ptr(input), input.len() as size_t);
    let ret_val = from_buf (raw_ptr, 32);
    csha256_destroy (raw_ptr);
    ret_val
  }
}


/* BYTESTRING HELPER FUNCTIONS */

pub fn push_u32_le (mut buf: ~[u8], val: u32) -> ~[u8]
{
  buf.push ((val) as u8);
  buf.push ((val >> 8) as u8);
  buf.push ((val >> 16) as u8);
  buf.push ((val >> 24) as u8);
  return buf;
}

pub fn push_u64_le (mut buf: ~[u8], val: u64) -> ~[u8]
{
  buf.push ((val) as u8);
  buf.push ((val >> 8) as u8);
  buf.push ((val >> 16) as u8);
  buf.push ((val >> 24) as u8);
  buf.push ((val >> 32) as u8);
  buf.push ((val >> 40) as u8);
  buf.push ((val >> 48) as u8);
  buf.push ((val >> 56) as u8);
  return buf;
}

pub fn push_vi_le (mut buf: ~[u8], val: u64) -> ~[u8]
{
  match val {
    0..0xfc => {
      buf.push (val as u8);
    }
    0xfd..0xffff => {
      buf.push (0xfd as u8);
      buf.push ((val) as u8);
      buf.push ((val >> 8) as u8);
    }
    0x10000..0xffffffff => {
      buf.push (0xfe as u8);
      return push_u32_le (buf, val as u32);
    }
    _ => {
      buf.push (0xff as u8);
      return push_u64_le (buf, val);
    }
  }
  return buf;
}


/**
 * Trait for hashable things (analogous to Serialize* in bitcoind)
 */
pub trait Hashable {
  fn to_hash(&self) -> ~[u8];
}

