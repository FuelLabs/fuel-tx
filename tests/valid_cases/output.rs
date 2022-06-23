use fuel_tx::*;
use rand::rngs::StdRng;
use rand::{Rng, RngCore, SeedableRng};

#[test]
fn coin() {
    let mut rng_base = StdRng::seed_from_u64(8586);
    let rng = &mut rng_base;

    Output::coin(rng.gen(), rng.next_u64(), rng.gen())
        .validate(1, &[])
        .unwrap();
}

#[test]
fn contract() {
    let mut rng_base = StdRng::seed_from_u64(8586);
    let rng = &mut rng_base;

    Output::contract(1, rng.gen(), rng.gen())
        .validate(
            2,
            &[
                Input::coin_signed(
                    rng.gen(),
                    rng.gen(),
                    rng.next_u64(),
                    rng.gen(),
                    rng.next_u32().to_be_bytes()[0],
                    rng.next_u64(),
                ),
                Input::contract(rng.gen(), rng.gen(), rng.gen(), rng.gen()),
            ],
        )
        .unwrap();

    let err = Output::contract(0, rng.gen(), rng.gen())
        .validate(
            2,
            &[
                Input::coin_signed(
                    rng.gen(),
                    rng.gen(),
                    rng.next_u64(),
                    rng.gen(),
                    rng.next_u32().to_be_bytes()[0],
                    rng.next_u64(),
                ),
                Input::contract(rng.gen(), rng.gen(), rng.gen(), rng.gen()),
            ],
        )
        .err()
        .unwrap();

    assert_eq!(ValidationError::OutputContractInputIndex { index: 2 }, err);

    let err = Output::contract(2, rng.gen(), rng.gen())
        .validate(
            2,
            &[
                Input::coin_signed(
                    rng.gen(),
                    rng.gen(),
                    rng.next_u64(),
                    rng.gen(),
                    rng.next_u32().to_be_bytes()[0],
                    rng.next_u64(),
                ),
                Input::contract(rng.gen(), rng.gen(), rng.gen(), rng.gen()),
            ],
        )
        .err()
        .unwrap();

    assert_eq!(ValidationError::OutputContractInputIndex { index: 2 }, err);
}

#[test]
fn message() {
    let mut rng_base = StdRng::seed_from_u64(8586);
    let rng = &mut rng_base;

    Output::message(Address::zeroed(), 0)
        .validate(1, &[])
        .expect("expected valid zeroed output");

    let err = Output::message(rng.gen(), 0)
        .validate(1, &[])
        .err()
        .expect("expected invalid output with recipient");

    assert_eq!(
        ValidationError::OutputMessageRecipientNotZero { index: 1 },
        err
    );

    let err = Output::message(Address::zeroed(), rng.gen())
        .validate(1, &[])
        .err()
        .expect("expected invalid output with amount");

    assert_eq!(
        ValidationError::OutputMessageAmountNotZero { index: 1 },
        err
    );
}

#[test]
fn change() {
    let mut rng_base = StdRng::seed_from_u64(8586);
    let rng = &mut rng_base;

    Output::change(rng.gen(), rng.next_u64(), rng.gen())
        .validate(1, &[])
        .unwrap();
}

#[test]
fn variable() {
    let mut rng_base = StdRng::seed_from_u64(8586);
    let rng = &mut rng_base;

    Output::variable(rng.gen(), rng.next_u64(), rng.gen())
        .validate(1, &[])
        .unwrap();
}

#[test]
fn contract_created() {
    let mut rng_base = StdRng::seed_from_u64(8586);
    let rng = &mut rng_base;

    Output::contract_created(rng.gen(), rng.gen())
        .validate(1, &[])
        .unwrap();
}
