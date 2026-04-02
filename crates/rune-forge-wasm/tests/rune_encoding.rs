use rune_forge_wasm::{create_rune, decode_rune_base64, verify_rune};

#[test]
fn create_and_decode_roundtrip() {
    let secret = "0000000000000000000000000000000000000000000000000000000000000000";
    let restrictions = "method=listfunds";

    let base64_rune = create_rune(secret, restrictions).unwrap();
    assert!(!base64_rune.is_empty());

    let decoded_json = decode_rune_base64(&base64_rune).unwrap();
    let decoded: serde_json::Value = serde_json::from_str(&decoded_json).unwrap();
    let alts = &decoded["restrictions"][0]["alternatives"];
    assert_eq!(alts[0]["field"], "method");
    assert_eq!(alts[0]["op"], "=");
    assert_eq!(alts[0]["value"], "listfunds");
}

#[test]
fn create_rune_multiple_restrictions() {
    let secret = "0000000000000000000000000000000000000000000000000000000000000000";
    let restrictions = "method=listfunds|method=xpay&pnameamount_msat<1000000001";

    let base64_rune = create_rune(secret, restrictions).unwrap();
    let decoded_json = decode_rune_base64(&base64_rune).unwrap();
    let decoded: serde_json::Value = serde_json::from_str(&decoded_json).unwrap();
    let restrictions_arr = decoded["restrictions"].as_array().unwrap();
    assert_eq!(restrictions_arr.len(), 2);
}

#[test]
fn create_rune_invalid_secret() {
    let result = create_rune("not-hex", "method=listfunds");
    assert!(result.is_err());
}

#[test]
fn decode_rune_base64_invalid_input() {
    let result = decode_rune_base64("not-a-valid-rune!!!");
    assert!(result.is_err());
}

#[test]
fn verify_rune_valid() {
    let secret = "0000000000000000000000000000000000000000000000000000000000000000";
    let base64_rune = create_rune(secret, "method=listfunds").unwrap();
    let result = verify_rune(secret, &base64_rune);
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn verify_rune_wrong_secret() {
    let secret = "0000000000000000000000000000000000000000000000000000000000000000";
    let wrong_secret = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
    let base64_rune = create_rune(secret, "method=listfunds").unwrap();
    let result = verify_rune(wrong_secret, &base64_rune);
    assert!(result.is_err());
}
