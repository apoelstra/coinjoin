
use std::rand;
use std::rand::Rng;

use transaction::{Transaction, TxIn, TxOut};
use util;

/**/
fn match_input(in1: &TxIn, in2: &TxIn) -> bool
{
  /* We don't check scriptSig since that will be different
   * for different transactions. */
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
 * Merge signed transactions
 * This function verifies that all the transactions are the same modulo
 * signatures, then combines them all into one big transaction that has
 * all the available signatures in place.
 */
pub fn merge_signed_transactions (txlist: &[Transaction]) -> Option<Transaction>
{
  if txlist.len() == 0 { return None; }

  /* The first transaction will be our ``master'' list of inputs and outputs.
   * Every other transaction needs to match this or else that's a failure.
   */
  let mut master = txlist[0].clone();
  /* Rust needs me to copy the master transaction's hash here, since
   * I later mutate the master (by adding signatures) and it fears this
   * will somehow bojangle the call to .to_hash(). */
  let master_hash = util::u8_to_hex_string (master.to_hash());

  /* Loop through all transactions, merging onto master */
  for tx in txlist.iter() {
    /* Check that version and locktime at least match */
    if tx.nVersion != master.nVersion {
      println (format! ("Tx {:s} did not match {:s} (version {:u} vs {:u})!",
        util::u8_to_hex_string (master.to_hash()),
        util::u8_to_hex_string (tx.to_hash()),
        master.nVersion, tx.nVersion));
      return None;
    }
    if tx.nLockTime != master.nLockTime {
      println (format! ("Tx {:s} did not match {:s} (locktime {:u} vs {:u})!",
        util::u8_to_hex_string (master.to_hash()),
        util::u8_to_hex_string (tx.to_hash()),
        master.nLockTime, tx.nLockTime));
      return None;
    }

    /* Check that outputs match */
    for (tx1, tx2) in tx.output.iter().zip(master.output.iter()) {
      if !match_output (tx1, tx2) {
        println (format! ("Tx {:s} did not match {:s} (output {:s}:{:u} vs {:s}:{:u})!",
          master_hash,
          util::u8_to_hex_string (tx.to_hash()),
          util::u8_to_hex_string (tx1.scriptPubKey), tx1.nValue,
          util::u8_to_hex_string (tx2.scriptPubKey), tx2.nValue));
        return None;
      }
    }

    /* Check that inputs match -- if they do, and a signature exists, take it */
    for (tx1, tx2) in tx.input.iter().zip(master.input.mut_iter()) {
      if match_input (tx1, tx2) {
        if tx1.scriptSig.len() > 0 {
          tx2.scriptSig = tx1.scriptSig.clone();
        }
      } else {
        println (format! ("Tx {:s} did not match {:s} (input {:s}:{:u} vs {:s}:{:u})!",
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

/**
 * Merge unsigned transactions
 * This function takes a bunch of transactions and creates a new, big
 * transaction with all the inputs and outputs from the originals, but
 * no signatures. It also randomizes the ordering.
 */
pub fn merge_unsigned_transactions (txlist: &[Transaction]) -> Option<Transaction>
{
  if txlist.len() == 0 { return None; }

  /* The first transaction will be our ``master'' list of inputs and outputs.
   * Every other transaction needs to match this or else that's a failure.
   */
  let mut master = Transaction {
    nVersion: txlist[0].nVersion,
    nLockTime: txlist[0].nLockTime,
    input: ~[], output: ~[]
  };

  /* Loop through all transactions, merging onto master */
  for tx in txlist.iter() {
    /* Check that version and locktime match, because otherwise it's unclear
     * what to do. (I guess it doesn't matter, in principle some humans will
     * verify this before it gets signed..) */
    if tx.nVersion != master.nVersion {
      println (format! ("Tx {:s} did not match {:s} (version {:u} vs {:u})!",
        util::u8_to_hex_string (master.to_hash()),
        util::u8_to_hex_string (tx.to_hash()),
        master.nVersion, tx.nVersion));
      return None;
    }
    if tx.nLockTime != master.nLockTime {
      println (format! ("Tx {:s} did not match {:s} (locktime {:u} vs {:u})!",
        util::u8_to_hex_string (master.to_hash()),
        util::u8_to_hex_string (tx.to_hash()),
        master.nLockTime, tx.nLockTime));
      return None;
    }

    /* Pile all the outputs on */
    for tx in tx.output.iter() {
      master.output.push (tx.clone());
    }

    /* Check for duplicate inputs and bail otherwise. This is pretty-much
     * guaranteed to be a mistake. (Probably there are also duplicate outputs,
     * but those are legal, so I don't want to delete them.) POLS says we
     * crash. */
    for tx in tx.input.iter() {
      for tx_dup in master.input.iter() {
        if match_input (tx, tx_dup) {
          println (format! ("Duplicate input {:s}:{:u} detected. Cowardly refusing to merge.",
            util::u8_to_hex_string (tx.prev_hash), tx.prev_index));
          return None;
        }
      }
      let mut new_tx = tx.clone();
      new_tx.scriptSig = ~[];     /* Delete any existing signatures */
      master.input.push (new_tx);
    }
  }

  /* Randomize the inputs and outputs */
  let mut rng = rand::task_rng();
  rng.shuffle_mut (master.input);
  rng.shuffle_mut (master.output);

  Some(master)
}



