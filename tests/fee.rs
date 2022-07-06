use fuel_tx::{ConsensusParameters, PanicReason, TransactionFee, Word};

const PARAMS: ConsensusParameters = ConsensusParameters::DEFAULT
    .with_gas_per_byte(2)
    .with_gas_price_factor(3);

#[test]
fn base_fee_is_calculated_correctly() {
    let metered_bytes = 5;
    let gas_limit = 7;
    let gas_price = 11;

    let fee = TransactionFee::from_values(&PARAMS, metered_bytes, gas_limit, gas_price)
        .expect("failed to calculate fee")
        .into();

    let expected = PARAMS.gas_per_byte * metered_bytes + gas_limit;
    let expected = expected * gas_price;
    let expected = expected as f64 / PARAMS.gas_price_factor as f64;
    let expected = expected.ceil() as Word;

    assert_eq!(expected, fee);
}

#[test]
fn base_fee_ceils() {
    let metered_bytes = 5;
    let gas_limit = 7;
    let gas_price = 11;

    let fee = TransactionFee::from_values(&PARAMS, metered_bytes, gas_limit, gas_price)
        .expect("failed to calculate fee")
        .into();

    let expected = PARAMS.gas_per_byte * metered_bytes + gas_limit;
    let expected = expected * gas_price;
    let expected = expected as f64 / PARAMS.gas_price_factor as f64;
    let truncated = expected as Word;
    let expected = expected.ceil() as Word;

    assert_ne!(truncated, expected);
    assert_eq!(expected, fee);
}

#[test]
fn base_fee_zeroes() {
    let metered_bytes = 5;
    let gas_limit = 7;
    let gas_price = 0;

    let fee = TransactionFee::from_values(&PARAMS, metered_bytes, gas_limit, gas_price)
        .expect("failed to calculate fee")
        .into();

    let expected = 0u64;

    assert_eq!(expected, fee);
}

#[test]
fn base_fee_wont_overflow_on_bytes() {
    let metered_bytes = Word::MAX;
    let gas_limit = 7;
    let gas_price = 11;

    let err = TransactionFee::from_values(&PARAMS, metered_bytes, gas_limit, gas_price)
        .expect_err("expected overflow error");

    assert_eq!(PanicReason::ArithmeticOverflow, err);
}

#[test]
fn base_fee_wont_overflow_on_limit() {
    let metered_bytes = 5;
    let gas_limit = Word::MAX;
    let gas_price = 11;

    let err = TransactionFee::from_values(&PARAMS, metered_bytes, gas_limit, gas_price)
        .expect_err("expected overflow error");

    assert_eq!(PanicReason::ArithmeticOverflow, err);
}

#[test]
fn base_fee_wont_overflow_on_price() {
    let metered_bytes = 5;
    let gas_limit = 7;
    let gas_price = Word::MAX;

    let err = TransactionFee::from_values(&PARAMS, metered_bytes, gas_limit, gas_price)
        .expect_err("expected overflow error");

    assert_eq!(PanicReason::ArithmeticOverflow, err);
}
