use rfb_labs_week_2::{InputKind, OutPoint, OutputType, Transaction, TxOutput};

fn regular_input(value: u64) -> InputKind {
    InputKind::Regular {
        previous_output: OutPoint {
            txid: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            vout: 0,
        },
        value,
        sequence: u32::MAX,
    }
}

fn output(value: u64, recipient: &str) -> TxOutput {
    TxOutput {
        value,
        recipient: recipient.into(),
        output_type: OutputType::P2wpkh,
    }
}

// These tests are ignored so the starter repository builds before students
// implement the TODOs. Remove `#[ignore]` from one test at a time while working.

#[test]
fn valid_regular_transaction_passes_validation() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(120_000));
    transaction.add_output(output(90_000, "bc1qreceiver"));
    transaction.add_output(output(28_000, "bc1qsender"));

    assert_eq!(transaction.validate(), Ok(()));
    assert_eq!(transaction.total_input_value(), 120_000);
    assert_eq!(transaction.total_output_value(), 118_000);
    assert_eq!(transaction.fee(), Ok(2_000));
}

#[test]
fn outputs_cannot_exceed_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_output(output(60_000, "bc1qreceiver"));

    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::OutputsExceedInputs {
            total_inputs: 50_000,
            total_outputs: 60_000,
        })
    );
}

#[test]
fn test_highest_value_and_recipient_filtering() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(200_000));

    let out1 = output(50_000, "alice");
    let out2 = output(120_000, "bob");
    let out3 = output(20_000, "alice");

    transaction.add_output(out1);
    transaction.add_output(out2);
    transaction.add_output(out3);

    let highest = rfb_labs_week_2::highest_value_output(&transaction).unwrap();
    assert_eq!(highest.value, 120_000);
    assert_eq!(highest.recipient, "bob");

    let alice_outputs = rfb_labs_week_2::find_outputs_for_recipient(&transaction, "alice");
    assert_eq!(alice_outputs.len(), 2);
    assert_eq!(alice_outputs[0].value, 50_000);
    assert_eq!(alice_outputs[1].value, 20_000);
}

#[test]
fn valid_coinbase_transaction_passes_validation() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Coinbase {
        block_height: 100_000,
        reward: 625_000_000,
    });
    transaction.add_output(output(625_000_000, "bc1qminer"));

    assert_eq!(transaction.validate(), Ok(()));
    assert_eq!(transaction.total_input_value(), 625_000_000);
    assert_eq!(transaction.total_output_value(), 625_000_000);
    assert_eq!(transaction.fee(), Ok(0));
}

#[test]
fn reject_no_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_output(output(10_000, "bc1qreceiver"));
    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::NoInputs)
    );
}

#[test]
fn reject_no_outputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(10_000));
    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::NoOutputs)
    );
}

#[test]
fn reject_zero_value_non_op_return_output() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_output(TxOutput {
        value: 0,
        recipient: "bc1qreceiver".into(),
        output_type: OutputType::P2wpkh,
    });
    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::ZeroValueOutput)
    );
}

#[test]
fn accept_zero_value_op_return_output() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_output(TxOutput {
        value: 0,
        recipient: "".into(),
        output_type: OutputType::OpReturn,
    });
    transaction.add_output(output(48_000, "bc1qchange"));
    assert_eq!(transaction.validate(), Ok(()));
}

#[test]
fn reject_mixed_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(regular_input(50_000));
    transaction.add_input(InputKind::Coinbase {
        block_height: 12345,
        reward: 50_000,
    });
    transaction.add_output(output(90_000, "bc1qreceiver"));
    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::CoinbaseMixedWithRegularInputs)
    );
}

#[test]
fn reject_multiple_coinbase_inputs() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Coinbase {
        block_height: 12345,
        reward: 50_000,
    });
    transaction.add_input(InputKind::Coinbase {
        block_height: 12346,
        reward: 50_000,
    });
    transaction.add_output(output(100_000, "bc1qreceiver"));
    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::MultipleCoinbaseInputs)
    );
}

#[test]
fn reject_empty_txid() {
    let mut transaction = Transaction::new(2, 0);
    transaction.add_input(InputKind::Regular {
        previous_output: OutPoint {
            txid: "".into(),
            vout: 0,
        },
        value: 50_000,
        sequence: u32::MAX,
    });
    transaction.add_output(output(48_000, "bc1qreceiver"));
    assert_eq!(
        transaction.validate(),
        Err(rfb_labs_week_2::TransactionError::InvalidTxid)
    );
}
