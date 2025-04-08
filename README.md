# fuel-sp1

## Install Dependencies

- Rust MSRV 1.85.0
   ```
    rustup toolchain install 1.85.0
    ```

- sp1 toolchain
    ```
    curl -L https://sp1up.succinct.xyz | bash
    sp1up --version 4.1.3
    ```

## Repository structure

Since each proving game should have its own entrypoint, it lives as its own crate.
We have `fuel-proving-games-sp1` which aims to aggregate these and provides the following features -

1. If imported as a library, you may access helpers to generate and verify proofs for the associated proving game.
2. If executed as a binary, you may run proof generation & verification via CLI (upcoming).

## Run proving tests

```
cargo test -p fuel-proving-games-sp1 prove_all_fixtures_and_collect_report
```

Make sure you use the correct env vars for the specific prover.

For CUDA proving, use the following feature flag:
```
SP1_PROVER=cuda cargo test -p fuel-proving-games-sp1 prove_all_fixtures_and_collect_report --features cuda
```

see [here](https://docs.succinct.xyz/docs/sp1/generating-proofs/hardware-acceleration/cuda#usage) for cuda instructions specific to sp1.

## Run execution tests

```
cargo test -p fuel-proving-games-sp1 run_all_fixtures_and_collect_report
```
