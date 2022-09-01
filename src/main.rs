use clap::Parser;
use csv::WriterBuilder;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::{collections::HashMap, error::Error, fs::File, path::PathBuf};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(value_parser)]
    file: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
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

// should use the referenced transaction
fn dispute(client: &mut Client, transaction: &Transaction) {
    client.hold(transaction.amount.unwrap());
}

// should use the referenced transaction
fn resolve(client: &mut Client, transaction: &Transaction) {
    client.release(transaction.amount.unwrap());
}

// should use the referenced transaction
fn chargeback(client: &mut Client, transaction: &Transaction) {
    client.withdraw(transaction.amount.unwrap());
    client.freeze()
}

#[derive(Serialize, Deserialize, Debug)]
struct Transaction {
    #[serde(rename(deserialize = "type"))]
    transaction_type: TransactionType,
    client: u16,
    tx: u32,

    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    amount: Option<Decimal>,
}

fn handle_transaction(transaction: Transaction, client_list: &mut HashMap<u16, Client>) {
    if !client_list.contains_key(&transaction.client) {
        client_list.insert(transaction.client, Client::new(transaction.client));
    };
    let client = client_list.get_mut(&transaction.client).unwrap();

    match transaction.transaction_type {
        TransactionType::Deposit => client.deposit(transaction.amount.unwrap()),
        TransactionType::Withdrawl => client.withdraw(transaction.amount.unwrap()),
        TransactionType::Dispute => dispute(client, &transaction),
        TransactionType::Resolve => resolve(client, &transaction),
        TransactionType::ChargeBack => chargeback(client, &transaction),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let file = File::open(args.file)?;

    let mut rdr = csv::Reader::from_reader(file);
    let mut client_list: HashMap<u16, Client> = HashMap::new();

    for result in rdr.deserialize() {
        let transaction: Transaction = result?;
        handle_transaction(transaction, &mut client_list);
    }

    let handle = io::stdout().lock();
    let mut wtr = WriterBuilder::new().from_writer(handle);
    for ele in client_list.into_values() {
        wtr.serialize(ele).unwrap();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_transaction_deposit_test() {
        let mut client_list: HashMap<u16, Client> = HashMap::new();

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
        );

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
        );

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
}
