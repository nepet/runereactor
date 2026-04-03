use rune_reactor::compiler::compile;
use rune_reactor::parser::parse_policy;
use rune_reactor::types::Op;

#[test]
fn test_monitoring_rf() {
    let input = include_str!("../../../examples/monitoring.rf");
    let policy = parse_policy(input).unwrap();
    let rune = compile(&policy).unwrap();

    // allow + global = 2 restrictions
    assert_eq!(rune.restrictions.len(), 2);

    // Method whitelist: 3 methods (^list, ^get, summary)
    assert_eq!(rune.restrictions[0].alternatives.len(), 3);
    assert_eq!(rune.restrictions[0].alternatives[0].field, "method");
    assert_eq!(rune.restrictions[0].alternatives[0].op, Op::StartsWith);
    assert_eq!(rune.restrictions[0].alternatives[0].value, "list");
    assert_eq!(rune.restrictions[0].alternatives[1].op, Op::StartsWith);
    assert_eq!(rune.restrictions[0].alternatives[1].value, "get");
    assert_eq!(rune.restrictions[0].alternatives[2].op, Op::Eq);
    assert_eq!(rune.restrictions[0].alternatives[2].value, "summary");

    // global: method / listdatastore
    assert_eq!(rune.restrictions[1].alternatives.len(), 1);
    assert_eq!(rune.restrictions[1].alternatives[0].field, "method");
    assert_eq!(rune.restrictions[1].alternatives[0].op, Op::Ne);
    assert_eq!(rune.restrictions[1].alternatives[0].value, "listdatastore");
}

#[test]
fn test_payments_rf() {
    let input = include_str!("../../../examples/payments.rf");
    let policy = parse_policy(input).unwrap();
    let rune = compile(&policy).unwrap();

    // allow + 2 when-xpay restrictions = 3 restrictions
    assert_eq!(rune.restrictions.len(), 3);

    // Method whitelist: 3 exact methods
    assert_eq!(rune.restrictions[0].alternatives.len(), 3);
    assert!(rune.restrictions[0]
        .alternatives
        .iter()
        .all(|c| c.field == "method" && c.op == Op::Eq));

    // when xpay: method/xpay | pnameamount_msat<100000000 | pnameamount_msat!
    assert_eq!(rune.restrictions[1].alternatives.len(), 3);
    assert_eq!(rune.restrictions[1].alternatives[0].op, Op::Ne);
    assert_eq!(rune.restrictions[1].alternatives[0].value, "xpay");
    assert_eq!(rune.restrictions[1].alternatives[1].field, "pnameamount_msat");
    assert_eq!(rune.restrictions[1].alternatives[1].op, Op::Lt);
    assert_eq!(rune.restrictions[1].alternatives[2].op, Op::Missing);

    // when xpay: method/xpay | rate=10
    assert_eq!(rune.restrictions[2].alternatives.len(), 2);
    assert_eq!(rune.restrictions[2].alternatives[0].value, "xpay");
    assert_eq!(rune.restrictions[2].alternatives[1].field, "rate");
    assert_eq!(rune.restrictions[2].alternatives[1].op, Op::Eq);
    assert_eq!(rune.restrictions[2].alternatives[1].value, "10");
}

#[test]
fn test_operator_rf() {
    let input = include_str!("../../../examples/operator.rf");
    let policy = parse_policy(input).unwrap();
    let rune = compile(&policy).unwrap();

    // id + 2 tags + allow + fundchannel + xpay(2) + close + global = 9 restrictions
    assert_eq!(rune.restrictions.len(), 9);

    // id
    assert_eq!(rune.restrictions[0].alternatives[0].field, "id");
    assert_eq!(rune.restrictions[0].alternatives[0].op, Op::Eq);

    // 2 tags
    assert_eq!(rune.restrictions[1].alternatives[0].op, Op::Comment);
    assert_eq!(rune.restrictions[1].alternatives[0].field, "role");
    assert_eq!(rune.restrictions[2].alternatives[0].op, Op::Comment);
    assert_eq!(rune.restrictions[2].alternatives[0].field, "version");

    // allow methods: ^list, getinfo, fundchannel, close, xpay (5 methods)
    assert_eq!(rune.restrictions[3].alternatives.len(), 5);
    assert_eq!(rune.restrictions[3].alternatives[0].op, Op::StartsWith);
    assert_eq!(rune.restrictions[3].alternatives[0].value, "list");
    assert_eq!(rune.restrictions[3].alternatives[1].op, Op::Eq);
    assert_eq!(rune.restrictions[3].alternatives[1].value, "getinfo");

    // when fundchannel: method/fundchannel | pnameamount<1000001
    assert_eq!(rune.restrictions[4].alternatives[0].value, "fundchannel");
    assert_eq!(rune.restrictions[4].alternatives[1].field, "pnameamount");
    assert_eq!(rune.restrictions[4].alternatives[1].op, Op::Lt);

    // when xpay: (A or B) and C → 2 restrictions
    assert_eq!(rune.restrictions[5].alternatives.len(), 3);
    assert_eq!(rune.restrictions[5].alternatives[0].value, "xpay");
    assert_eq!(rune.restrictions[6].alternatives.len(), 2);
    assert_eq!(rune.restrictions[6].alternatives[0].value, "xpay");
    assert_eq!(rune.restrictions[6].alternatives[1].field, "rate");

    // when close: method/close | pnamedestination=bc1q...
    assert_eq!(rune.restrictions[7].alternatives[0].value, "close");
    assert_eq!(rune.restrictions[7].alternatives[1].field, "pnamedestination");

    // global: per=1min
    assert_eq!(rune.restrictions[8].alternatives[0].field, "per");
    assert_eq!(rune.restrictions[8].alternatives[0].value, "1min");
}
