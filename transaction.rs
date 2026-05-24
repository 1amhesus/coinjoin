
use std::hashmap::HashMap;
use std::to_str::ToStr;

use decoder;
use util;
use hash;

pub struct TxIn {
  prev_hash: ~[u8],
  prev_index: u32,
  scriptSig: ~[u8],
  nSequence: u32,
  nHashType: u8
}

pub struct TxOut {
  nValue: u64,
  scriptPubKey: ~[u8]
}

pub struct Transaction {
  nVersion: u32,
  nLockTime: u32,
  input: ~[TxIn],
  output: ~[TxOut]
}

/**
 * 16진수 문자열 파서 상태 머신
 */
enum ParserState {
  ReadVersion,
  ReadInputCount,
  ReadTxinHash,
  ReadTxinIndex,
  ReadTxinScriptSigLen,
  ReadTxinScriptSig,
  ReadTxinSequence,
  ReadOutputCount,
  ReadTxoutValue,
  ReadTxoutScriptLen,
  ReadTxoutScript,
  ReadLockTime,
  Error,
  Done
}

/**
 * 빈 TxIn/TxOut 생성자
 */
fn new_blank_txin() -> TxIn
{
  TxIn { prev_hash: ~[], prev_index: 0, scriptSig: ~[], nSequence: 0, nHashType: 0 }
}

fn new_blank_txout() -> TxOut
{
  TxOut { nValue: 0, scriptPubKey: ~[] }
}

/**
 * 복사 생성자
 */
impl Clone for TxOut {
  fn clone(&self) -> TxOut
  {
    TxOut { nValue: self.nValue, scriptPubKey: self.scriptPubKey.clone() }
  }
}

impl Clone for TxIn {
  fn clone(&self) -> TxIn
  {
    TxIn {
      prev_hash: self.prev_hash.clone(),
      prev_index: self.prev_index,
      scriptSig: self.scriptSig.clone(),
      nSequence: self.nSequence,
      nHashType: self.nHashType
    }
  }
}

impl Clone for Transaction {
  fn clone(&self) -> Transaction
  {
    Transaction {
      nVersion: self.nVersion,
      nLockTime: self.nLockTime,
      input: self.input.clone(),
      output: self.output.clone()
    }
  }
}



/**
 * 생성자 / createrawtransaction 파서
 */
pub fn from_hex (hex_string: &[u8]) -> Option<Transaction>
{
  let mut rv: Transaction = Transaction {
    nVersion: 0,
    nLockTime: 0,
    input: ~[],
    output: ~[]
  };

  /* 보조 상태 */
  let mut width = 0;
  let mut vin_counter: u64 = 0;
  let mut vout_counter: u64 = 0;

  /* 상태 머신 실행 */
  let mut iter = hex_string.iter();
  let mut state = ReadVersion;  /* 초기 상태: 버전 읽기 */
  loop {
    state = match state {
      /* 빅엔디언 u32 버전 읽기 */
      ReadVersion => {
        match decoder::decode_token (&mut iter, decoder::Unsigned32) {
          decoder::Integer(n) => { rv.nVersion = n as u32; ReadInputCount }
          _ => Error
        }
      }
      /* 입력 읽기 */
      ReadInputCount => {
        match decoder::decode_token (&mut iter, decoder::VarInt) {
          decoder::Integer(0) => { Error }  /* 입력이 0개이면 실패 */
          decoder::Integer(n) => { vin_counter = n; ReadTxinHash }
          _ => Error
        }
      }
      /* txin의 해시 읽기 */
      ReadTxinHash => {
        match decoder::decode_token (&mut iter, decoder::Bytestring(32)) {
          decoder::String(s) => {
            let mut new_txin = new_blank_txin();
            new_txin.prev_hash = s;
            rv.input.push (new_txin);
            ReadTxinIndex
          }
          _ => Error
        }
      }
      /* txin의 인덱스 읽기 */
      ReadTxinIndex => {
        match decoder::decode_token (&mut iter, decoder::Unsigned32) {
          decoder::Integer(n) => { rv.input[rv.input.len() - 1].prev_index = n as u32; ReadTxinScriptSigLen }
          _ => Error
        }
      }
      /* txin의 scriptSig 읽기 */
      ReadTxinScriptSigLen => {
        match decoder::decode_token (&mut iter, decoder::VarInt) {
          decoder::Integer(0) => { ReadTxinSequence }  /* 길이가 0이면 scriptSig 건너뛰기 */
          decoder::Integer(n) => { width = n; ReadTxinScriptSig }
          _ => Error
        }
      }
      ReadTxinScriptSig => {
        match decoder::decode_token (&mut iter, decoder::Bytestring(width)) {
          decoder::String(s) => {
            /* 표준 tx scriptSig는 PUSH[n+1] 뒤에 n바이트 서명과
             * 1바이트 해시 타입이 따라온다. 우리에게 의미가 명확한 형태가
             * 사실상 이것뿐이라 이 형식을 하드코딩하며,
             * 더 지능적인 처리를 해도 실익이 없다. */
            if s[0] > 0 && s[0] < 76 && s.len() > s[0] as uint {
              rv.input[rv.input.len() - 1].nHashType = s[s[0]];
            }
            rv.input[rv.input.len() - 1].scriptSig = s;
            ReadTxinSequence
          }
          _ => Error
        }
      }
      /* txin의 시퀀스 번호 읽기 */
      ReadTxinSequence => {
        match decoder::decode_token (&mut iter, decoder::Unsigned32) {
          decoder::Integer(n) => {
            rv.input[rv.input.len() - 1].nSequence = n as u32;
            vin_counter -= 1;
            if vin_counter > 0 {
              ReadTxinHash
            } else {
              ReadOutputCount
            }
          }
          _ => Error
        }
      }
      /* 출력 읽기 */
      ReadOutputCount => {
        match decoder::decode_token (&mut iter, decoder::VarInt) {
          decoder::Integer(0) => { Error }  /* 출력이 0개이면 실패 (꼭 그래야 하는지는 미정) */
          decoder::Integer(n) => { vout_counter = n; ReadTxoutValue }
          _ => Error
        }
      }
      /* txout 값 읽기 */
      ReadTxoutValue => {
        match decoder::decode_token (&mut iter, decoder::Unsigned64) {
          decoder::Integer(n) => {
            let mut new_output = new_blank_txout();
            new_output.nValue = n;
            rv.output.push (new_output);
            ReadTxoutScriptLen
          }
          _ => Error
        }
      }
      /* txout 스크립트 읽기 */
      ReadTxoutScriptLen => {
        match decoder::decode_token (&mut iter, decoder::VarInt) {
          /* 길이가 0이면 scriptPubKey 건너뛰기 */
          decoder::Integer(0) => {
            vout_counter -= 1;
            if vout_counter > 0 {
              ReadTxoutValue
            } else {
              ReadLockTime
            }
          }
          decoder::Integer(n) => { width = n; ReadTxoutScript }
          _ => Error
        }
      }
      ReadTxoutScript => {
        match decoder::decode_token (&mut iter, decoder::Bytestring(width)) {
          decoder::String(s) => {
            rv.output[rv.output.len() - 1].scriptPubKey = s;
            vout_counter -= 1;
            if vout_counter > 0 {
              ReadTxoutValue
            } else {
              ReadLockTime
            }
          }
          _ => Error
        }
      }
      /* 출력 처리 완료, nLockTime 읽기 */
      ReadLockTime => {
        match decoder::decode_token (&mut iter, decoder::Unsigned32) {
          decoder::Integer(n) => { rv.nLockTime = n as u32; Done }
          _ => Error
        }
      }
      /* 완료 */
      Error => { return None; }
      Done => { break }
    }
  }

  Some (rv)
}

impl Transaction {
/**
 * 비공개 직렬화 함수
 */
  fn serialize (&self) -> ~[u8]
  {
    let mut rv:~[u8] = ~[];

    /* 버전 푸시 */
    rv = hash::push_u32_le (rv, self.nVersion);
    /* txins 푸시 */
    rv = hash::push_vi_le (rv, self.input.len() as u64);
    for txin in self.input.iter() {
      rv.push_all (txin.prev_hash);
      rv = hash::push_u32_le (rv, txin.prev_index);
      rv = hash::push_vi_le (rv, txin.scriptSig.len() as u64);
      rv.push_all (txin.scriptSig);
      rv = hash::push_u32_le (rv, txin.nSequence);
    }
    /* txouts 푸시 */
    rv = hash::push_vi_le (rv, self.output.len() as u64);
    for txout in self.output.iter() {
      rv = hash::push_u64_le (rv, txout.nValue);
      rv = hash::push_vi_le (rv, txout.scriptPubKey.len() as u64);
      rv.push_all (txout.scriptPubKey);
    }
    /* locktime 푸시 */
    rv = hash::push_u32_le (rv, self.nLockTime);
    rv
  }

  /** mpo 게터 */
  pub fn most_popular_output (&self) -> u64 {
    fn fold_function ((max_elem, max_count): (u64, uint), (&elem, &count): (&u64, &uint)) -> (u64, uint) {
      if count > max_count {
        (elem, count)
      } else if count < max_count {
        (max_elem, max_count)
      } else if elem == 0 && max_elem == 0 {
        (0, count)  /* 이 경우는 발생하면 안 된다 */
      } else {
        let mut max_scan  = max_elem;
        let mut elem_scan = elem;
        /* 동률이면 더 둥근 수를 선택 */
        while (max_scan % 10) == 0 &&
              (elem_scan % 10) == 0 {
          max_scan /= 10;
          elem_scan /= 10;
        }
        if max_scan % 10 == 0 { (max_elem, max_count) } else { (elem, count) }
      }
    };

    let mut values: HashMap<u64,uint> = HashMap::new ();
    /* 각 출력을 순회하며 개수 증가 */
    for output in self.output.iter() {
      values.mangle (output.nValue, (), |_,_| 1, |_,v,_| { *v += 1; });
    }
    values.iter().fold ((0, 0), fold_function).first()
  }

  /** mpo 개수 게터 */
  pub fn most_popular_output_count (&self) -> uint {
    let mut mpo_count = 0;
    let mpo = self.most_popular_output ();
    for output in self.output.iter() {
      if output.nValue == mpo {
        mpo_count += 1;
      }
    }
    mpo_count
  }
}

impl hash::Hashable for Transaction {
  /**
   * 이 함수는 트랜잭션의 txid를 생성한다.
   */
  fn to_hash(&self) -> ~[u8]
  {
    /* TXID는 직렬화 데이터의 SHA256^2 값이다. bitcoin이 이를
     * 리틀엔디언 256비트 수로 다루므로 순서를 뒤집는다.  */
    let mut rv = hash::sha256_sum (hash::sha256_sum (self.serialize()));
    rv.reverse();
    rv
  }
}

impl ToStr for Transaction {
  fn to_str(&self) -> ~str
  {
    util::u8_to_hex_string (self.serialize())
  }
}

