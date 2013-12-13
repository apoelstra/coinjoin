
use std::to_str::ToStr;

use decoder;
use util;
use hash;

pub struct TxIn {
  prev_hash: ~[u8],
  prev_index: u32,
  scriptSig: ~[u8],
  nSequence: u32
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
 * Hex string parser state machine
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
 * Constructor for empty TxIn/TxOut
 */
fn new_blank_txin() -> TxIn
{
  TxIn { prev_hash: ~[], prev_index: 0, scriptSig: ~[], nSequence: 0 }
}

fn new_blank_txout() -> TxOut
{
  TxOut { nValue: 0, scriptPubKey: ~[] }
}

/**
 * Copy constructors
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
      nSequence: self.nSequence
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
 * Constructor / createrawtransaction parser
 */
pub fn from_hex (hex_string: &[u8]) -> Option<Transaction>
{
  let mut rv: Transaction = Transaction { nVersion: 0, nLockTime: 0, input: ~[], output: ~[] };

  /* Auxiallary state */
  let mut width = 0;
  let mut vin_counter: u64 = 0;
  let mut vout_counter: u64 = 0;

  /* RUN STATE MACHINE */
  let iter = &mut hex_string.iter() as &mut Iterator<&u8>;
  let mut state = ReadVersion;  /* Initial state: read version */
  /* Is there a nicer way to compare C-like enums? */
  while (state as int) != (Done as int) &&
        (state as int) != (Error as int) {
    state = match state {
      /* Read big-endian u32 version */
      ReadVersion => {
        match decoder::decode_token (iter, decoder::Unsigned32) {
          decoder::Integer(n) => { rv.nVersion = n as u32; ReadInputCount }
          decoder::String(_) => Error,
          decoder::Invalid => Error
        }
      }
      /* READ INPUTS */
      ReadInputCount => {
        match decoder::decode_token (iter, decoder::VarInt) {
          decoder::Integer(0) => { Error }  /* zero inputs is a failure */
          decoder::Integer(n) => { vin_counter = n; ReadTxinHash }
          decoder::String(_) => Error,
          decoder::Invalid => Error
        }
      }
      /* Read the hash of a txin */
      ReadTxinHash => {
        match decoder::decode_token (iter, decoder::Bytestring(32)) {
          decoder::Integer(_) => Error,
          decoder::String(s) => {
            let mut new_txin = new_blank_txin();
            new_txin.prev_hash = s;
            rv.input.push (new_txin);
            ReadTxinIndex
          }
          decoder::Invalid => Error
        }
      }
      /* Read the index of a txin */
      ReadTxinIndex => {
        match decoder::decode_token (iter, decoder::Unsigned32) {
          decoder::Integer(n) => { rv.input[rv.input.len() - 1].prev_index = n as u32; ReadTxinScriptSigLen }
          decoder::String(_) => Error,
          decoder::Invalid => Error
        }
      }
      /* Read the scriptSig of a txin */
      ReadTxinScriptSigLen => {
        match decoder::decode_token (iter, decoder::VarInt) {
          decoder::Integer(0) => { ReadTxinSequence }  /* skip scriptSig if it has width 0 */
          decoder::Integer(n) => { width = n; ReadTxinScriptSig }
          decoder::String(_) => Error,
          decoder::Invalid => Error
        }
      }
      ReadTxinScriptSig => {
        match decoder::decode_token (iter, decoder::Bytestring(width)) {
          decoder::Integer(_) => Error,
          decoder::String(s) => {
            rv.input[rv.input.len() - 1].scriptSig = s;
            ReadTxinSequence
          }
          decoder::Invalid => Error
        }
      }
      /* Read the sequence no. of a txin */
      ReadTxinSequence => {
        match decoder::decode_token (iter, decoder::Unsigned32) {
          decoder::Integer(n) => {
            rv.input[rv.input.len() - 1].nSequence = n as u32;
            if vin_counter > 0 {
              vin_counter -= 1;
              ReadTxinHash
            } else {
              ReadOutputCount
            }
          }
          decoder::String(_) => Error,
          decoder::Invalid => Error
        }
      }
      /* READ OUTPUTS */
      ReadOutputCount => {
        match decoder::decode_token (iter, decoder::VarInt) {
          decoder::Integer(0) => { Error }  /* zero outputs is a failure (maybe it shouldn't be?) */
          decoder::Integer(n) => { vout_counter = n; ReadTxoutValue }
          decoder::String(_) => Error,
          decoder::Invalid => Error
        }
      }
      /* Read txout value */
      ReadTxoutValue => {
        match decoder::decode_token (iter, decoder::Unsigned32) {
          decoder::Integer(n) => {
            let mut new_output = new_blank_txout();
            new_output.nValue = n;
            rv.output.push (new_output);
            ReadTxoutScriptLen
          }
          decoder::String(_) => Error,
          decoder::Invalid => Error
        }
      }
      /* Read txout script */
      ReadTxoutScriptLen => {
        match decoder::decode_token (iter, decoder::VarInt) {
          /* skip scriptPubKey if it has width 0 */
          decoder::Integer(0) => {
            if vout_counter > 0 {
              vout_counter -= 1;
              ReadTxoutValue
            } else {
              ReadLockTime
            }
          }
          decoder::Integer(n) => { width = n; ReadTxoutScript }
          decoder::String(_) => Error,
          decoder::Invalid => Error
        }
      }
      ReadTxoutScript => {
        match decoder::decode_token (iter, decoder::Bytestring(width)) {
          decoder::Integer(_) => Error,
          decoder::String(s) => {
            rv.output[rv.output.len() - 1].scriptPubKey = s;
            if vout_counter > 0 {
              vout_counter -= 1;
              ReadTxoutValue
            } else {
              ReadLockTime
            }
          }
          decoder::Invalid => Error
        }
      }
      /* DONE OUTPUTS, Read nLockTime */
      ReadLockTime => {
        match decoder::decode_token (iter, decoder::Unsigned32) {
          decoder::Integer(n) => { rv.nLockTime = n as u32; ReadInputCount }
          decoder::String(_) => Error,
          decoder::Invalid => Error
        }
      }
      /* Finished */
      Error => { break }
      Done => { break }
    }
  }

  if (state as int) == (Done as int) {
    Some (rv)
  } else {
    None
  }
}

/**
 * Private serialize function
 */
impl Transaction {
  fn serialize (&self) -> ~[u8]
  {
    let mut rv:~[u8] = ~[];

    /* push version */
    rv = hash::push_u32_le (rv, self.nVersion);
    /* push txins */
    rv = hash::push_vi_le (rv, self.input.len() as u64);
    for txin in self.input.iter() {
      rv.push_all (txin.prev_hash);
      rv = hash::push_u32_le (rv, txin.prev_index);
      rv = hash::push_vi_le (rv, txin.scriptSig.len() as u64);
      rv.push_all (txin.scriptSig);
      rv = hash::push_u32_le (rv, txin.nSequence);
    }
    /* push txouts */
    rv = hash::push_vi_le (rv, self.output.len() as u64);
    for txout in self.output.iter() {
      rv = hash::push_u64_le (rv, txout.nValue);
      rv = hash::push_vi_le (rv, txout.scriptPubKey.len() as u64);
      rv.push_all (txout.scriptPubKey);
    }
    /* push locktime */
    rv = hash::push_u32_le (rv, self.nLockTime);
    rv
  }
}

impl hash::Hashable for Transaction {
  /**
   * This function generates a txid for the transaction.
   */
  fn to_hash(&self) -> ~[u8]
  {
    /* The TXID is the SHA256^2 of the serialization. We reverse it since bitcoin
     * treats it as a little-endian 256-bit number.  */
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


