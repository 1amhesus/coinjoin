
use std::libc::size_t;
use std::vec::from_buf;


#[link(name = "sha-wrapper")]
#[link(name = "crypto")]
extern {
  fn csha256_sum (input: *u8, len: size_t) -> *u8;
  fn csha256_destroy (input: *u8);
}

/* 해시 함수 */

/**
 * 원시 비트열의 SHA256 합을 계산한다
 */
pub fn sha256_sum (input: &[u8]) -> ~[u8]
{
  unsafe {
    let raw_ptr = csha256_sum (input.as_ptr(), input.len() as size_t);
    let ret_val = from_buf (raw_ptr, 32);
    csha256_destroy (raw_ptr);
    ret_val
  }
}


/* 바이트열 보조 함수 */

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
 * 해시 가능한 대상용 트레이트 (bitcoind의 Serialize*와 유사)
 */
pub trait Hashable {
  fn to_hash(&self) -> ~[u8];
}

