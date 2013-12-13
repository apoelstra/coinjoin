
use hash::Hashable;

/* Modules */
mod decoder;
mod hash;
mod merge_signed;
mod transaction;
mod util;

/**
 * Entry point
 */
fn main()
{
  println ("Welcome to coinjoin-merger-signed. Enter each partially-signed raw transaction");
  println ("on a separate line, followed by a blank line or EOF to finish.");

  let mut transactions: ~[transaction::Transaction] = ~[];

  let mut next_ln = util::read_hex();
  while next_ln.len() > 0 {
    match transaction::from_hex (next_ln) {
      Some(t) => { transactions.push (t); }
      None => {}
    }
    next_ln = util::read_hex();
  }

  match merge_signed::merge_signed_transactions (transactions) {
    None => { println ("Failed to merge transactions."); }
    Some(t) => { println (format! ("{:s}", t.to_str())); }
  }
}

