use bitcoin::Network;
use rfb_labs_week_6::raw_demo::run_raw_script_demo;

#[test]
fn test_raw_script_demo_execution() {
    let report = run_raw_script_demo(Network::Regtest).unwrap();

    assert!(!report.script_hex.is_empty());
    assert!(!report.spending_raw_hex.is_empty());
    assert_eq!(report.witness_items.len(), 3); // [sig, preimage, witness_script]
    assert!(!report.explanation.is_empty());
}
