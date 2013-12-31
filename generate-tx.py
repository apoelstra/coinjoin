
# Note - to use this script you need Jeff Garzik's python-bitcoinrpc
# https://github.com/jgarzik/python-bitcoinrpc

import sys;
import json;
from bitcoinrpc.authproxy import AuthServiceProxy;

# SET THESE VALUES
rpc_user = "bitcoinrpc";
rpc_pass = "[ENTER YOUR PASSWORD HERE]";
rpc_host = "localhost";
rpc_port = 8332;

donation_minimum = 0;
donation_per_input = 3000;
donation_address = "1ForFeesAndDonationsSpendHerdtWbWy";


def to_satoshi(s):
  return int (100000000 * float (s));
def from_satoshi(s):
  return float (s) / 100000000;


if len(sys.argv) < 3:
  print ("Usage: %s <input size> <target output size in BTC>" % sys.argv[0]);
  exit (0);

service = AuthServiceProxy ("http://%s:%s@%s:%d" % (rpc_user, rpc_pass, rpc_host, rpc_port));

balance = to_satoshi (service.getbalance());
unspent = service.listunspent();
target_in  = to_satoshi (sys.argv[1]);
target_out = to_satoshi (sys.argv[2]);

if balance < target_in:
  print ("Cannot spend %f; only have %f in wallet." % (from_satoshi (target_in), from_satoshi (balance)));
  exit (0);

if target_out > target_in:
  print ("Please have a smaller target output than input value.");
  exit (0);


# FIND INPUTS
# TODO: have a smarter coin selection algo
# For now we just sort the coins by increasing abs(value - target output), then select in order
inputs = [];
donation = 0;
total_in = 0;

unspent.sort (key=lambda coin: abs(to_satoshi (coin['amount']) - target_in));

for coin in unspent:
  total_in += to_satoshi (coin['amount']);
  donation += donation_per_input;
  inputs.append (dict (txid = coin['txid'], vout = coin['vout']));
  if total_in > target_in:
    break;

if donation < donation_minimum:
  donation = donation_minimum;

# FIND OUTPUTS
outputs = dict ();
outputs[donation_address] = from_satoshi (donation);
total_in -= donation;
while total_in > target_out:
  outputs[service.getnewaddress()] = from_satoshi (target_out);
  total_in -= target_out;
outputs[service.getnewaddress()] = from_satoshi (total_in);

# Make the transaction
print service.createrawtransaction (inputs, outputs);





