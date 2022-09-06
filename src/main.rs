use anyhow::anyhow;
use anyhow::Context;
use anyhow::Ok;
use anyhow::Result;
use clap::Parser;
use csv::WriterBuilder;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::io;
use std::{collections::HashMap, fs::File, path::PathBuf};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(value_parser)]
    file: PathBuf,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
enum TransactionType {
    Deposit,
    Withdrawl,
    Dispute,
    Resolve,
    ChargeBack,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
struct Client {
    #[serde(rename(serialize = "client"))]
    id: u16,

    #[serde(rename(serialize = "available"))]
    available_amount: Decimal,

    #[serde(rename(serialize = "held"))]
    held_amount: Decimal,

    #[serde(rename(serialize = "total"))]
    total_amount: Decimal,

    locked: bool,
}

impl Client {
    fn new(id: u16) -> Self {
        Self {
            id,
            available_amount: dec!(0),
            held_amount: dec!(0),
            total_amount: dec!(0),
            locked: false,
        }
    }

    // increases available and total funds by amount
    fn deposit(&mut self, amount: Decimal) {
        self.available_amount += amount;
        self.total_amount += amount;
    }

    // decreases available and total funds by amount
    fn withdraw(&mut self, amount: Decimal) {
        self.available_amount -= amount;
        self.total_amount -= amount;
    }

    // available funds should decrease by amount,
    //    held should increase by amount.
    // total should remain the same
    fn hold(&mut self, amount: Decimal) {
        self.available_amount -= amount;
        self.held_amount += amount;
    }

    // held funds should decrease by the amount
    // available funds should increase by the maount
    // total should remain the same
    fn release(&mut self, amount: Decimal) {
        self.held_amount -= amount;
        self.available_amount += amount;
    }

    fn freeze(&mut self) {
        self.locked = true;
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct Transaction {
    #[serde(rename(deserialize = "type"))]
    transaction_type: TransactionType,
    client: u16,
    tx: u32,

    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    amount: Option<Decimal>,
}

impl Transaction {
    fn amount(&self) -> Result<Decimal> {
        match self.amount {
            Some(amount) => Ok(amount),
            None => Err(anyhow!("No amount field in Transaction: {:?}", self)),
        }
    }
}

type TransactionList = Vec<Transaction>;
type ClientList = HashMap<u16, Client>;

fn handle_transaction(
    transaction: Transaction,
    client_list: &mut ClientList,
    // refactor to hashmap
    transaction_list: &mut TransactionList,
) -> Result<()> {
    if !client_list.contains_key(&transaction.client) {
        client_list.insert(transaction.client, Client::new(transaction.client));
    };

    if !transaction_list.contains(&transaction) {
        transaction_list.push(transaction.clone());
    }

    // This should never error, since we instered the client above
    let client = client_list.get_mut(&transaction.client).unwrap();

    match transaction.transaction_type {
        TransactionType::Deposit => {
            client.deposit(transaction.amount().context("Deposit type transaction")?)
        }
        TransactionType::Withdrawl => {
            client.withdraw(transaction.amount().context("Withdrawl type transaction")?)
        }
        TransactionType::Dispute => {
            if let Some(target_transaction) = transaction_list
                .iter()
                .find(|tran| tran.tx == transaction.tx)
            {
                client.hold(
                    target_transaction
                        .amount()
                        .context("Targeted from Dispute transaction")?,
                )
            };
        }
        TransactionType::Resolve => {
            if let Some(target_transaction) = transaction_list
                .iter()
                .find(|tran| tran.tx == transaction.tx)
            {
                client.release(
                    target_transaction
                        .amount()
                        .context("Targeted from Resolve transaction")?,
                )
            };
        }
        TransactionType::ChargeBack => {
            if let Some(target_transaction) = transaction_list
                .iter()
                .find(|tran| tran.tx == transaction.tx)
            {
                client.withdraw(
                    target_transaction
                        .amount()
                        .context("Targeted from chargeback transaction")?,
                );
                client.freeze();
            };
        }
    };
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    let file = File::open(args.file)?;

    let mut rdr = csv::Reader::from_reader(file);
    let mut client_list: ClientList = HashMap::new();
    let mut transaction_list: TransactionList = vec![];

    for result in rdr.deserialize() {
        let transaction: Transaction = result?;
        handle_transaction(transaction, &mut client_list, &mut transaction_list)?;
    }

    let handle = io::stdout().lock();
    let mut wtr = WriterBuilder::new().from_writer(handle);
    for ele in client_list.into_values() {
        wtr.serialize(ele)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_transaction_deposit_test() {
        let mut client_list: ClientList = HashMap::new();
        let mut transaction_list: TransactionList = vec![];

        let client_id = 1;

        let transaction_amount = dec!(10.4752);

        handle_transaction(
            Transaction {
                client: client_id,
                transaction_type: TransactionType::Deposit,
                tx: 1,
                amount: Some(transaction_amount),
            },
            &mut client_list,
            &mut transaction_list,
        )
        .unwrap();

        assert_eq!(
            &Client {
                id: client_id,
                available_amount: transaction_amount,
                held_amount: dec!(0),
                total_amount: transaction_amount,
                locked: false,
            },
            client_list.get(&client_id).unwrap()
        );

        handle_transaction(
            Transaction {
                client: client_id,
                transaction_type: TransactionType::Deposit,
                tx: 1,
                amount: Some(dec!(5.0000)),
            },
            &mut client_list,
            &mut transaction_list,
        )
        .unwrap();

        assert_eq!(
            &Client {
                id: client_id,
                available_amount: transaction_amount + dec!(5),
                held_amount: dec!(0),
                total_amount: transaction_amount + dec!(5),
                locked: false,
            },
            client_list.get(&client_id).unwrap()
        );
    }

    #[test]
    fn client_deposit() {
        let mut client = Client::new(1);
        let amount = dec!(10);
        client.deposit(amount);
        assert_eq!(client.available_amount, amount);
        assert_eq!(client.total_amount, amount);
    }

    #[test]
    fn client_withdraw() {
        let mut client = Client::new(1);
        client.deposit(dec!(15));
        client.withdraw(dec!(7));
        assert_eq!(client.available_amount, dec!(8));
        assert_eq!(client.total_amount, dec!(8));
    }

    // available funds should decrease by amount,
    //    held should increase by amount.
    // total should remain the same
    #[test]
    fn client_hold() {
        let mut client = Client::new(1);
        client.deposit(dec!(15));
        client.hold(dec!(5));
        assert_eq!(client.available_amount, dec!(10));
        assert_eq!(client.total_amount, dec!(15));
        assert_eq!(client.held_amount, dec!(5));
    }

    // held funds should decrease by the amount
    // available funds should increase by the maount
    // total should remain the same
    #[test]
    fn client_release() {
        let mut client = Client::new(1);
        client.deposit(dec!(20));
        client.hold(dec!(10));
        client.release(dec!(5));
        assert_eq!(client.available_amount, dec!(15));
        assert_eq!(client.total_amount, dec!(20));
        assert_eq!(client.held_amount, dec!(5));
    }

    #[test]
    fn client_freeze() {
        let mut client = Client::new(1);
        client.freeze();
        assert!(client.locked);
    }

    #[test]
    fn dispute_should_() {
        unimplemented!();
    }
}
