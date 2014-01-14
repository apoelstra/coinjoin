#!/bin/sh

TESTDIR=tests
UNSIGNED=./coinjoin-merge-unsigned
SIGNED=./coinjoin-merge-signed

process_output()
{
  local prog="$1"
  local input="$2"
  local ln

  # Run the program output through an awk script which
  # simply filters for the actual data output and echos
  # it in a well-defined order.
  "$prog" < "$input" | awk '
  function despace(s) {
    gsub(/[[:space:]]*/, "", s);
    return s;
  }

  BEGIN {
    while (getline) {
      split ($0, fields, ":");
      switch (despace(fields[1])) {
      case "mpo":
        mpo = despace(fields[2]);
        break;
      case "mpc":
        mpc = despace(fields[2]);
        break;
      case "hex":
        hex = despace(fields[2]);
        break;
      case "err":
        err = despace(fields[2]);
        break;
      }
    }
    print "mpo:", mpo;
    print "mpc:", mpc;
    print "hex:", hex;
    print "err:", err;
  }'
}



# Run unsigned tests
for suite in $TESTDIR/unsigned/*
do
  for run in $suite/*.input
  do
    if [[ -f $run ]]
    then
      echo -n "$UNSIGNED: Running $run... ";
      outf=$(echo $run | sed 's/input$/output/')
      expf=$(echo $run | sed 's/input$/expected/')
      if [[ -f "$expf" ]]
      then
        process_output $UNSIGNED "$run" > "$outf"
        diff -q "$expf" "$outf" > /dev/null
        if [[ "$?" == "0" ]]
        then echo "success."
        else
          echo "failed."
          echo "Diff output:"
          diff "$expf" "$outf"
        fi
        rm $outf
      elif [[ -e "$expf" ]]
      then
        echo "failed (expected output file not an ordinary file)."
      else
        echo "failed (no expected output file)."
      fi
    fi
  done
done

