use decodetrx::decode_transaction;
use serde_json::Value;

#[test]
fn test_decode_segwit_transaction() {
    let hex_tx = "0200000000010196277c04c986c1ad78c909287fd12dba2924324699a0232e0533f46a6a3916bb0100000000ffffffff026400000000000000160014274ae586ad2035efb4c25049c155f98310d7e106ca16440000000000160014599bcef6387256c6b019030c421b4a4d382fe2600247304402204d94a1e4047ca38a450177ccb6f88585ca147f1939df343d8ac5d962c5f35bb302206f7fa42c21c47ebccdc460393d35c5dfd3b6f0a26cf10fac23d3e6fab71835c20121020cb972a66e3fb1cdcc9efcad060b4457ebec534942700d4af1c0d82a33aa13f100000000";

    let json_str = decode_transaction(hex_tx.to_string()).unwrap();
    let json: Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(json["version"], 2);
    assert_eq!(json["lock_time"], 0);
    assert_eq!(json["inputs"].as_array().unwrap().len(), 1);
    assert_eq!(json["outputs"].as_array().unwrap().len(), 2);

    // Assert outputs amounts in BTC
    let outputs = json["outputs"].as_array().unwrap();
    assert_eq!(outputs[0]["amount"], 0.000001); // 100 sats = 0.000001 BTC
    assert_eq!(outputs[1]["amount"], 0.04462282); // 5700 sats = 0.04462282 BTC
}

#[test]
fn test_decode_legacy_transaction() {
    // Legacy P2PKH transaction
    let hex_tx = "01000000017b1eabe0209b1fe794124575ef807057c77ada2138ae4fa8d6c4de0398a14f3f000000008b4830450221008949f0a31a353ebac226d5efb952f59bafec0ceb72a4c4613ee596700b0e00ce022079776997bc9d476de6e674154126b8047ab5a299e1211e402a4b3d756d87ac03014104c06d0b5433a0b0a03332c918a514d3f3f0f7f90f42df1e51f048d2c49d68d1847e5b562145b2f0c78a05c3b9b47e248b6bc168b449176bc5ee86ec9e31d45c60ffffffff0140420f00000000001976a9142e00b21a8d052601ee34d0b7a8d0032e3a1f9d5f88ac00000000";

    let json_str = decode_transaction(hex_tx.to_string()).unwrap();
    let json: Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(json["version"], 1);
    assert_eq!(json["lock_time"], 0);
    assert_eq!(json["inputs"].as_array().unwrap().len(), 1);
    assert_eq!(json["outputs"].as_array().unwrap().len(), 1);

    assert!(json["transaction_id"].is_string());
}
