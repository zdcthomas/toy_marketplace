Project spec

## Details
Given a CSV representing a series of transactions, implement a simple toy
transactions engine that processes the payments crediting and debiting
accounts. After processing the complete set of payments output the client
account balances You should be able to run your payments engine like

```sh
 cargo run -- transactions.csv > accounts.csv
```
The input file is the first and only argument to the binary. Output should be
written to std out 

### Input
The input will be a CSV file with the columns type, client, tx, and amount. You
can assume the type is a string, the client column is a valid u16 client ID,
the tx is a valid u32 transaction ID, and the amount is a decimal value with a
precision of up to four places past the decimal. For example

```csv
type, client, tx, amount
deposit, 1, 1, 1.0
deposit, 2, 2, 2.0
deposit, 1, 3, 2.0
withdrawal, 1, 4, 1.5
withdrawal, 2, 5, 3.0
```

The client ID will be unique per client though are not guaranteed to be
ordered. Transactions to the client account 2 could occur before transactions
to the client account 1. Likewise, transaction IDs (tx) are globally unique,
though are also not guaranteed to be ordered. You can assume the transactions
occur chronologically in the file, so if transaction b appears after a in the
input file then you can assume b occurred chronologically after a. Whitespaces
and decimal precisions (up to four places past the decimal) must be accepted by
your program. 

## Output

The output should be a list of client IDs (client), available amounts
(available), held amounts (held), total amounts (total), and whether the
account is locked (locked). Columns are defined
as
| Column    | Description                                                                                  |
|-----------|----------------------------------------------------------------------------------------------|
| available | The total funds that are available for trading, staking,                                     |
|           | withdrawal, etc. This should be equal to the total - held amounts                            |
|-----------|----------------------------------------------------------------------------------------------|
| held      | The total funds that are held for dispute. This should be equal to total - available amounts |
|-----------|----------------------------------------------------------------------------------------------|
| total     | The total funds that are available or held. This should be equal to available + held         |
|-----------|----------------------------------------------------------------------------------------------|
| locked    | Whether the account is locked. An account is locked if a charge back occurs                  |
|-----------|----------------------------------------------------------------------------------------------|

For example
```csv
client, available, held, total, locked
1, 1.5, 0.0, 1.5, false
2, 2.0, 0.0, 2.0, false
```

Spacing and displaying decimals for round values do not matter. Row ordering
also does not matter. The above output will be considered the exact same as the
following.

```csv
client,available,held,total,
2,2,0,2,false
1,1.5,0,1.5,false
```

## Precision

You can assume a precision of `four places past the decimal` and should output values with the
same level of precision.

## Types of Transactions
### Deposit

A deposit is a credit to the client's asset account, meaning it should increase
the available and total funds of the client account

A deposit looks like

| type    | client | tx | amount |
|---------|--------|----|--------|
| deposit | 1      | 1  | 1.0    |
|---------|--------|----|--------|

### Withdrawal
A withdraw is a debit to the client's asset account, meaning it should decrease
the available and total funds of the client account

A withdrawal looks like

| type       | client | tx | amount |
|------------|--------|----|--------|
| withdrawal | 2      | 2  | 1.0    |
|------------|--------|----|--------|

If a client does not have sufficient available funds the withdrawal should fail
and the total amount of funds should not change

### Dispute

A dispute represents a client's claim that a transaction was erroneous and
should be reversed. The transaction shouldn't be reversed yet but the
associated funds should be held. This means that the clients available funds
should decrease by the amount disputed, their held funds should increase by the
amount disputed, while their total funds should remain the same.

A dispute looks like

| type    | client | tx | amount |
|---------|--------|----|--------|
| dispute | 1      | 1  |        |
|---------|--------|----|--------|

Notice that a dispute does not state the amount disputed. Instead a dispute
references the transaction that is disputed by ID. If the tx specified by the
dispute doesn't exist you can ignore it and assume this is an error on our
partners side.

### Resolve

A resolve represents a resolution to a dispute, releasing the associated held
funds. Funds that were previously disputed are no longer disputed. This means
that the clients held funds should decrease by the amount no longer disputed,
their available funds should increase by the amount no longer disputed, and
their total funds should remain the same.

A resolve looks like

| type    | client | tx | amount |
|---------|--------|----|--------|
| resolve | 1      | 1  |        |
|---------|--------|----|--------|

Like disputes, resolves do not specify an amount. Instead they refer to a
transaction that was under dispute by ID. If the tx specified doesn't exist, or
the tx isn't under dispute, you can ignore the resolve and assume this is an
error on our partner's side.

### Chargeback

A chargeback is the final state of a dispute and represents the client
reversing a transaction. Funds that were held have now been withdrawn. This
means that the clients held funds and total funds should decrease by the amount
previously disputed. If a chargeback occurs the client's account should be
immediately frozen.

A chargeback looks like

| type       | client | tx | amount |
|------------|--------|----|--------|
| chargeback | 1      | 1  |        |
|------------|--------|----|--------|

Like a dispute and a resolve a chargeback refers to the transaction by ID (tx)
and does not specify an amount. Like a resolve, if the tx specified doesn't
exist, or the tx isn't under dispute, you can ignore chargeback and assume this
is an error on our partner's side.
