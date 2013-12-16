
use std::hashmap::HashMap;
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
  let mut rv: Transaction = Transaction {
    nVersion: 0,
    nLockTime: 0,
    input: ~[],
    output: ~[]
  };

  /* Auxiallary state */
  let mut width = 0;
  let mut vin_counter: u64 = 0;
  let mut vout_counter: u64 = 0;

  /* RUN STATE MACHINE */
  let mut iter = hex_string.iter();
  let mut state = ReadVersion;  /* Initial state: read version */
  /* Is there a nicer way to compare C-like enums? */
  loop {
    state = match state {
      /* Read big-endian u32 version */
      ReadVersion => {
        match decoder::decode_token (&mut iter, decoder::Unsigned32) {
          decoder::Integer(n) => { rv.nVersion = n as u32; ReadInputCount }
          _ => Error
        }
      }
      /* READ INPUTS */
      ReadInputCount => {
        match decoder::decode_token (&mut iter, decoder::VarInt) {
          decoder::Integer(0) => { Error }  /* zero inputs is a failure */
          decoder::Integer(n) => { vin_counter = n; ReadTxinHash }
          _ => Error
        }
      }
      /* Read the hash of a txin */
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
      /* Read the index of a txin */
      ReadTxinIndex => {
        match decoder::decode_token (&mut iter, decoder::Unsigned32) {
          decoder::Integer(n) => { rv.input[rv.input.len() - 1].prev_index = n as u32; ReadTxinScriptSigLen }
          _ => Error
        }
      }
      /* Read the scriptSig of a txin */
      ReadTxinScriptSigLen => {
        match decoder::decode_token (&mut iter, decoder::VarInt) {
          decoder::Integer(0) => { ReadTxinSequence }  /* skip scriptSig if it has width 0 */
          decoder::Integer(n) => { width = n; ReadTxinScriptSig }
          _ => Error
        }
      }
      ReadTxinScriptSig => {
        match decoder::decode_token (&mut iter, decoder::Bytestring(width)) {
          decoder::String(s) => {
            rv.input[rv.input.len() - 1].scriptSig = s;
            ReadTxinSequence
          }
          _ => Error
        }
      }
      /* Read the sequence no. of a txin */
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
      /* READ OUTPUTS */
      ReadOutputCount => {
        match decoder::decode_token (&mut iter, decoder::VarInt) {
          decoder::Integer(0) => { Error }  /* zero outputs is a failure (maybe it shouldn't be?) */
          decoder::Integer(n) => { vout_counter = n; ReadTxoutValue }
          _ => Error
        }
      }
      /* Read txout value */
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
      /* Read txout script */
      ReadTxoutScriptLen => {
        match decoder::decode_token (&mut iter, decoder::VarInt) {
          /* skip scriptPubKey if it has width 0 */
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
      /* DONE OUTPUTS, Read nLockTime */
      ReadLockTime => {
        match decoder::decode_token (&mut iter, decoder::Unsigned32) {
          decoder::Integer(n) => { rv.nLockTime = n as u32; Done }
          _ => Error
        }
      }
      /* Finished */
      Error => { return None; }
      Done => { break }
    }
  }

  Some (rv)
}

impl Transaction {
/**
 * Private serialize function
 */
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

  /** Getter for mpo */
  pub fn most_popular_output (&self) -> u64 {
    fn fold_function ((max_elem, max_count): (u64, uint), (&elem, &count): (&u64, &uint)) -> (u64, uint) {
      if count > max_count {
        (elem, count)
      } else if count < max_count {
        (max_elem, max_count)
      } else if elem == 0 && max_elem == 0 {
        (0, count)  /* this shouldn't ever happen */
      } else {
        let mut max_scan  = max_elem;
        let mut elem_scan = elem;
        /* tiebreak goes to rounder number */
        while (max_scan % 10) == 0 &&
              (elem_scan % 10) == 0 {
          max_scan /= 10;
          elem_scan /= 10;
        }
        if max_scan % 10 == 0 { (max_elem, max_count) } else { (elem, count) }
      }
    };

    let mut values: HashMap<u64,uint> = HashMap::new ();
    /* For each output increment its count */
    for output in self.output.iter() {
      values.mangle (output.nValue, (), |_,_| 1, |_,v,_| { *v += 1; });
    }
    values.iter().fold ((0, 0), fold_function).first()
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


