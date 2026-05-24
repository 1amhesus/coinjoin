
use std::rand;
use std::rand::Rng;

use transaction::{Transaction, TxIn};
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

/**
 * 미서명 트랜잭션 병합
 * 이 함수는 여러 트랜잭션을 받아 새로운 큰 트랜잭션을 만들며,
 * 원본들의 모든 입력/출력을 포함하되
 * 서명은 제외한다. 또한 순서를 무작위화한다.
 */
pub fn merge_unsigned_transactions (txlist: &[Transaction]) -> Option<Transaction>
{
  if txlist.len() == 0 { return None; }

  /* 첫 번째 트랜잭션을 입력/출력의 ``마스터'' 목록으로 사용한다.
   * 나머지 트랜잭션은 이것과 일치해야 하며, 아니면 실패다.
   */
  let mut master = Transaction {
    nVersion: txlist[0].nVersion,
    nLockTime: txlist[0].nLockTime,
    input: ~[], output: ~[]
  };

  /* 모든 트랜잭션을 순회하며 마스터에 병합 */
  for tx in txlist.iter() {
    /* version과 locktime이 일치하는지 확인한다. 그렇지 않으면
     * 어떻게 처리할지 불명확하다 (서명 전 검증에 의존한다고 가정). */
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

    /* 모든 출력을 누적하고 중복 출력을 확인해 합산 */
    for tx in tx.output.iter() {
      let mut already_present = false;
      for tx_dup in master.output.mut_iter() {
        if tx_dup.scriptPubKey == tx.scriptPubKey {
          tx_dup.nValue += tx.nValue;
          already_present = true;
        }
      }
      if !already_present {
        master.output.push (tx.clone());
      }
    }

    /* 중복 입력이 있으면 중단한다. 거의 확실히 실수이기 때문이다.
     * (중복 출력도 있을 수 있지만 합법이므로 제거하지 않는다.) */
    for tx in tx.input.iter() {
      for tx_dup in master.input.iter() {
        if match_input (tx, tx_dup) {
          println (format! ("err: Duplicate input {:s}:{:u} detected. Cowardly refusing to merge.",
            util::u8_to_hex_string (tx.prev_hash), tx.prev_index));
          return None;
        }
      }
      let mut new_tx = tx.clone();
      /* 기존 서명은 제거하되 sighash 타입이 NONE|ANYONECANPAY인 경우는 예외다.
       * 병합 후에도 유효한 서명 타입이 사실상 이것뿐이기 때문이다.
       * (다만 CodeShark의 multisigner처럼 정보를 담는 경우는 지워질 수 있어
       * TODO: 향후 지원이 필요하다.) */
      if new_tx.nHashType != 0x82 {
        new_tx.scriptSig = ~[];
      }
      master.input.push (new_tx);
    }
  }

  /* 입력과 출력을 무작위화 */
  let mut rng = rand::task_rng();
  rng.shuffle_mut (master.input);
  rng.shuffle_mut (master.output);

  Some(master)
}


