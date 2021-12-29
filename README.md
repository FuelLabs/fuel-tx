# fuel-tx

[![build](https://github.com/FuelLabs/fuel-tx/actions/workflows/ci.yml/badge.svg)](https://github.com/FuelLabs/fuel-tx/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/fuel-tx?label=latest)](https://crates.io/crates/fuel-tx)
[![docs](https://docs.rs/fuel-tx/badge.svg)](https://docs.rs/fuel-tx/)
[![discord](https://img.shields.io/badge/chat%20on-discord-orange?&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.gg/xfpK4Pe)

Transaction implementation for the [FuelVM](https://github.com/FuelLabs/fuel-specs).

## Compile features

- `random`: Implement [rand](https://crates.io/crates/rand) features for the provided types.
- `std`: Unless set, the crate will link to the core-crate instead of the std-crate. More info [here](https://docs.rust-embedded.org/book/intro/no-std.html).
- `serde-types`: Add support for [serde](https://crates.io/crates/serde) for the types exposed by this crate.
- `serde-types-minimal`: Add support for `no-std` [serde](https://crates.io/crates/serde) for the types exposed by this crate.
