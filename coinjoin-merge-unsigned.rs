
use hash::Hashable;

/* Modules */
mod hash;
mod merge_unsigned;
mod transaction;
mod util;

/**
 * Entry point
 */
fn main()
{
  println ("Welcome to coinjoin-merger. Enter each unsigned raw transaction");
  println ("on a separate line, followed by a blank line or EOF to finish.");

  let mut transactions: ~[transaction::Transaction] = ~[];

  let mut next_ln = util::read_hex();
  while next_ln.len() > 0 {
    transactions.push (transaction::from_hex (next_ln));
    next_ln = util::read_hex();
  }

  match merge_unsigned::merge_unsigned_transactions (transactions) {
    None => { println ("Failed to merge transactions."); }
    Some(t) => { println (format! ("{:s}", t.to_str())); }
  }
}

