//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2::{InputKind, OutPoint, OutputType, Transaction, TxOutput};

fn main() {
    let mut transaction = Transaction::new(2, 0);

    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "7000000000000000000000000000000000000000000000000000000000000000".to_owned(),
            vout: 0,
        },
        value: 70_000,
        sequence: u32::MAX,
    });

    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "5000000000000000000000000000000000000000000000000000000000000000".to_owned(),
            vout: 0,
        },
        value: 50_000,
        sequence: u32::MAX,
    });

    transaction.add_output(TxOutput {
        value: 90_000,
        recipient: "bc1qreceiver".to_owned(),
        output_type: OutputType::P2wpkh,
    });

    transaction.add_output(TxOutput {
        value: 28_000,
        recipient: "bc1qsender".to_owned(),
        output_type: OutputType::P2wpkh,
    });

    match transaction.validate() {
        Ok(_) => println!("Transaction is valid!"),
        Err(e) => println!("Transaction is invalid: {}", e),
    }

    println!("{transaction}");
}
