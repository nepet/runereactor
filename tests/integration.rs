use rune_forge::compiler::compile;
use rune_forge::parser::parse_policy;
use rune_forge::types::Op;

#[test]
fn test_operator_rf() {
    let input = include_str!("../../rune-biscuit/examples/operator.rf");
    let policy = parse_policy(input).unwrap();
    let rune = compile(&policy).unwrap();

    // tag + allow + 2 when blocks = 4 restrictions
    assert_eq!(rune.restrictions.len(), 4);

    // Tag
    assert_eq!(rune.restrictions[0].alternatives[0].op, Op::Comment);
    assert_eq!(rune.restrictions[0].alternatives[0].field, "operator_id");

    // Method whitelist: 6 methods
    assert_eq!(rune.restrictions[1].alternatives.len(), 6);
    assert!(rune.restrictions[1]
        .alternatives
        .iter()
        .all(|c| c.field == "method" && c.op == Op::Eq));

    // when fundchannel: method/fundchannel | pnameamount<1000001
    assert_eq!(rune.restrictions[2].alternatives[0].op, Op::Ne);
    assert_eq!(rune.restrictions[2].alternatives[0].value, "fundchannel");
    assert_eq!(rune.restrictions[2].alternatives[1].field, "pnameamount");
    assert_eq!(rune.restrictions[2].alternatives[1].op, Op::Lt);

    // when xpay: method/xpay | pnameamount_msat<1000000001 | pnameamount_msat!
    assert_eq!(rune.restrictions[3].alternatives.len(), 3);
    assert_eq!(rune.restrictions[3].alternatives[0].op, Op::Ne);
    assert_eq!(rune.restrictions[3].alternatives[0].value, "xpay");
}

#[test]
fn test_advanced_rf() {
    let input = include_str!("../../rune-biscuit/examples/advanced.rf");
    let policy = parse_policy(input).unwrap();
    let rune = compile(&policy).unwrap();

    // id + 2 tags + allow + fundchannel + xpay(2 from AND) + close + global = 9 restrictions
    assert_eq!(rune.restrictions.len(), 9);

    // id=02abcdef...
    assert_eq!(rune.restrictions[0].alternatives[0].field, "id");
    assert_eq!(rune.restrictions[0].alternatives[0].op, Op::Eq);

    // 2 tags
    assert_eq!(rune.restrictions[1].alternatives[0].op, Op::Comment);
    assert_eq!(rune.restrictions[2].alternatives[0].op, Op::Comment);

    // allow methods (11 methods)
    assert_eq!(rune.restrictions[3].alternatives.len(), 11);

    // when fundchannel: 1 restriction
    assert_eq!(rune.restrictions[4].alternatives[0].value, "fundchannel");

    // when xpay: (A or B) and C → 2 restrictions
    // First: method/xpay | pnameamount_msat<1000000001 | pnameamount_msat!
    assert_eq!(rune.restrictions[5].alternatives.len(), 3);
    assert_eq!(rune.restrictions[5].alternatives[0].value, "xpay");
    // Second: method/xpay | rate=10
    assert_eq!(rune.restrictions[6].alternatives.len(), 2);
    assert_eq!(rune.restrictions[6].alternatives[0].value, "xpay");
    assert_eq!(rune.restrictions[6].alternatives[1].field, "rate");

    // when close: method/close | pnamedestination=bc1q...
    assert_eq!(rune.restrictions[7].alternatives[0].value, "close");
    assert_eq!(rune.restrictions[7].alternatives[1].field, "pnamedestination");

    // global: per=1min
    assert_eq!(rune.restrictions[8].alternatives[0].field, "per");
    assert_eq!(rune.restrictions[8].alternatives[0].op, Op::Eq);
    assert_eq!(rune.restrictions[8].alternatives[0].value, "1min");
}
