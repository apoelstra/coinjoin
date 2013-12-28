
use hash::Hashable;

/* Modules */
mod decoder;
mod hash;
mod merge_unsigned;
mod transaction;
mod util;

/**
 * Entry point
 */
fn main()
{
  println ("Welcome to coinjoin-merge-unsigned. Enter each unsigned raw transaction");
  println ("on a separate line, followed by a blank line or EOF to finish.");

  let mut transactions: ~[transaction::Transaction] = ~[];

  let mut next_ln = util::read_hex();
  while next_ln.len() > 0 {
    match transaction::from_hex (next_ln) {
      Some(t) => { transactions.push (t); }
      None => { println ("err: Failed to decode transaction."); }
    }
    next_ln = util::read_hex();
  }

  match merge_unsigned::merge_unsigned_transactions (transactions) {
    None => { println ("err: Failed to merge transactions."); }
    Some(t) => {
      println (format! ("mpo: {:f}", (t.most_popular_output() as f64) / 100000000f64 ));
      println (format! ("mpc: {:u}", t.most_popular_output_count()));
      println (format! ("hex: {:s}", t.to_str()));
    }
  }
}

