
use transaction::{Transaction, TxIn, TxOut};
use hash::Hashable;
use util;

/**/
fn match_input(in1: &TxIn, in2: &TxIn) -> bool
{
  /* scriptSig는 트랜잭션마다 달라질 수 있으므로 검사하지 않는다. */
  in1.prev_hash == in2.prev_hash &&
  in1.prev_index == in2.prev_index &&
  in1.nSequence == in2.nSequence
}

fn match_output(in1: &TxOut, in2: &TxOut) -> bool
{
  in1.nValue == in2.nValue &&
  in1.scriptPubKey == in2.scriptPubKey
}

/**
 * 서명된 트랜잭션 병합
 * 이 함수는 모든 트랜잭션이 서명을 제외하면 동일한지 검증하고,
 * 사용 가능한 서명을 모두 포함하는 하나의 큰 트랜잭션으로
 * 결합한다.
 */
pub fn merge_signed_transactions (txlist: &[Transaction]) -> Option<Transaction>
{
  if txlist.len() == 0 { return None; }

  /* 첫 번째 트랜잭션을 입력/출력의 ``마스터'' 목록으로 사용한다.
   * 나머지 트랜잭션은 이것과 일치해야 하며, 아니면 실패다.
   */
  let mut master = txlist[0].clone();
  /* Rust의 borrow checker 제약 때문에 마스터 트랜잭션 해시를 여기서 복사한다.
   * 이후 마스터를 변경(서명 추가)하므로 .to_hash() 호출이 꼬일 수 있다. */
  let master_hash = util::u8_to_hex_string (master.to_hash());

  /* 모든 트랜잭션을 순회하며 마스터에 병합 */
  for tx in txlist.iter() {
    /* 최소한 version과 locktime이 일치하는지 확인 */
    if tx.nVersion != master.nVersion {
      println (format! ("err: Tx {:s} did not match {:s} (version {:u} vs {:u})!",
        util::u8_to_hex_string (master.to_hash()),
        util::u8_to_hex_string (tx.to_hash()),
        master.nVersion, tx.nVersion));
      return None;
    }
    if tx.nLockTime != master.nLockTime {
      println (format! ("err: Tx {:s} did not match {:s} (locktime {:u} vs {:u})!",
        util::u8_to_hex_string (master.to_hash()),
        util::u8_to_hex_string (tx.to_hash()),
        master.nLockTime, tx.nLockTime));
      return None;
    }

    /* 출력이 일치하는지 확인 */
    for (tx1, tx2) in tx.output.iter().zip(master.output.iter()) {
      if !match_output (tx1, tx2) {
        println (format! ("err: Tx {:s} did not match {:s} (output {:s}:{:u} vs {:s}:{:u})!",
          master_hash,
          util::u8_to_hex_string (tx.to_hash()),
          util::u8_to_hex_string (tx1.scriptPubKey), tx1.nValue,
          util::u8_to_hex_string (tx2.scriptPubKey), tx2.nValue));
        return None;
      }
    }

    /* 입력이 일치하는지 확인하고, 서명이 있으면 채택 */
    for (tx1, tx2) in tx.input.iter().zip(master.input.mut_iter()) {
      if match_input (tx1, tx2) {
        if tx1.scriptSig.len() > 0 {
          tx2.scriptSig = tx1.scriptSig.clone();
        }
      } else {
        println (format! ("err: Tx {:s} did not match {:s} (input {:s}:{:u} vs {:s}:{:u})!",
          master_hash,
          util::u8_to_hex_string (tx.to_hash()),
          util::u8_to_hex_string (tx1.prev_hash), tx1.prev_index,
          util::u8_to_hex_string (tx2.prev_hash), tx2.prev_index));
        return None;
      }
    }
  }

  Some(master)
}
