# Toy marketplace challenge
See [project specification](./spec.md) for more details

## Arch decisions

### Mutative vs immutable functional design:
  Usually I strongly lean toward a more functional approach. Usually I
  would create a struct which tracks holds the current state of all
  clients/transactions and have the handle transaction function
  recursively move through this list, and ultimately return a final
  version of this state. However, for performance and lifetime reasons, I
  opted for a mutating-in-place strategy.

### To async, or not to async
  Try as I might, I could not find a good way to process this list
  more asynchronously, without running a first pass that groups all
  related transactions together. This felt silly.

### Not storing meta transactions
  I made a distinction between `standard transactions` (e.g deposit, withdrawl)
  and `meta transactions` (e.g dispute, chargeback, resolve). 

  This is primarily because meta transactions didn't have a meaningful
  `transaction_id`, and instead, use their `transaction_id` field to
  reference another transaction, so I would either have to store the
  transactions in a Vec and cause every transaction lookup to happen in
  O(n), or I could store just the standard transactions in a `HashMap`
  associated to their id. 

  I tried the former first, but the later turned out to make more sense
  when meta transactions can change the state of an existing transaction
  (i.e dispute them). 

  I don't love that transactions are mutable in this design. I'd rather
  they be completely immutable in an append only list, but keeping track
  of disputes would have added a bunch of complexity (probably in the form
  of an entire other data structure for the meta transactions).

  This is main decision I'd reevaluate if I had more time.

## If I had more time

### Test coverage
  Right now the test coverage is fairly spartan, and I'd like to shore it up
  more, especially with some larger integration tests.

### Remove nearly all expects
  There are a few truly impossible states in this app, and in some of
  these states (especially `None` values when retrieving from lists) I
  opted for simply expecting or unwrapping these values. But other times
  there are states that are only possible through function misuse, and
  these I should be bubbling up.
