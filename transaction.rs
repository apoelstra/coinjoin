
use std::to_str::ToStr;
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

  /* Initial state: read version, 4 bytes */
  let mut state = ReadVersion;
  let mut width = 4;
  let mut count = 0;

  let mut vin_counter = 0;
  let mut vout_counter = 0;
  let mut aux_width = 0;

  for ch in hex_string.iter() {
    count += 1;
    /* Run state machine */
    let next_state = match state {
      /* Read big-endian u32 version */
      ReadVersion => {
        width = 4;
        rv.nVersion += (*ch as u32) << ((count - 1) * 8);
        if count < width { ReadVersion } else { ReadInputCount }
      },
      /* READ INPUTS */
      ReadInputCount => {
        /* TODO: whenever rust gets stable macro support, this "read VI" code should be factored out */
        if count == 1 {
          width = match *ch {
            0xff => 8,
            0xfe => 4,
            0xfd => 2,
            n => { vin_counter = n as int; 0 }
          };
          if width > 0 { ReadInputCount }
          else if vin_counter == 0 { Error }
          else { ReadTxinHash }
        } else if count < width {
          vin_counter += (*ch as int) << ((count - 1) * 8);
          ReadInputCount
        } else {
          ReadTxinHash
        }
      },
      /* Read the hash of a txin */
      ReadTxinHash => {
        if count == 1 {
          vin_counter -= 1;
          width = 32;
          rv.input.push(new_blank_txin());
        }

        rv.input[rv.input.len() - 1].prev_hash.push (*ch);
        if count < width { ReadTxinHash } else { ReadTxinIndex }
      },
      /* Read the index of a txin */
      ReadTxinIndex => {
        width = 4;
        rv.input[rv.input.len() - 1].prev_index += (*ch as u32) << ((count - 1) * 8);
        if count < width { ReadTxinIndex } else { ReadTxinScriptSigLen }
      },
      /* Read the scriptSig of a txin */
      ReadTxinScriptSigLen => {
        if count == 1 {
          width = match *ch {
            0xff => 8,
            0xfe => 4,
            0xfd => 2,
            n => { aux_width = n as int; 0 }
          };
          if width > 0 { ReadTxinScriptSigLen }
          else if aux_width == 0 { ReadTxinSequence } /* skip sig if it has width 0 */
          else { ReadTxinScriptSig }
        } else if count < width {
          aux_width += (*ch as int) << ((count - 1) * 8);
          ReadTxinScriptSigLen
        } else {
          ReadTxinScriptSig
        }
      }
      ReadTxinScriptSig => {
        width = aux_width;
        rv.input[rv.input.len() - 1].scriptSig.push (*ch);
        if count < width { ReadTxinScriptSig } else { ReadTxinSequence }
      }
      /* Read the sequence no. of a txin */
      ReadTxinSequence => {
        width = 4;
        rv.input[rv.input.len() - 1].nSequence += (*ch as u32) << ((count - 1) * 8);
        if count < width { ReadTxinSequence }
        else if vin_counter > 0 { ReadTxinHash }
        else { ReadOutputCount }
      }
      /* READ OUTPUTS */
      ReadOutputCount => {
        if count == 1 {
          width = match *ch {
            0xff => 8,
            0xfe => 4,
            0xfd => 2,
            n => { vout_counter = n as int; 0 }
          };
          if width > 0 { ReadOutputCount }
          else if vout_counter == 0 { Error }
          else { ReadTxoutValue }
        } else if count < width {
          vout_counter += (*ch as int) << ((count - 1) * 8);
          ReadOutputCount
        } else {
          ReadTxoutValue
        }
      },
      /* Read txout value */
      ReadTxoutValue => {
        if count == 1 {
          vout_counter -= 1;
          width = 8;
          rv.output.push(new_blank_txout());
        }

        rv.output[rv.output.len() - 1].nValue += (*ch as u64) << ((count - 1) * 8);
        if count < width { ReadTxoutValue } else { ReadTxoutScriptLen }
      }
      /* Read txout script */
      ReadTxoutScriptLen => {
        if count == 1 {
          width = match *ch {
            0xff => 8,
            0xfe => 4,
            0xfd => 2,
            n => { aux_width = n as int; 0 }
          };
          if width > 0 { ReadTxoutScriptLen }
            /* skip script if it has width 0 */
          else if aux_width == 0 { if vout_counter > 0 { ReadTxoutValue } else { ReadLockTime } }
          else { ReadTxoutScript }
        } else if count < width {
          aux_width += (*ch as int) << ((count - 1) * 8);
          ReadTxoutScriptLen
        } else {
          ReadTxoutScript
        }
      } 
      ReadTxoutScript => {
        width = aux_width;
        rv.output[rv.output.len() - 1].scriptPubKey.push (*ch);
        if count < width { ReadTxoutScript }
        else if vout_counter > 0 { ReadTxoutValue }
        else { ReadLockTime }
      }
      /* DONE OUTPUTS, Read nLockTime */
      ReadLockTime => {
        width = 4;
        rv.nLockTime += (*ch as u32) << ((count - 1) * 8);
        if count < width { ReadLockTime } else { Done }
      }
      /* Error */
      Error => { break }
      /* End */
      Done => { break }
    };
    /* Increment counter, do state transition */
    if (next_state as int) != (state as int) { count = 0; }
    state = next_state;
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


