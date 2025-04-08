//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can have an
//! EVM-Compatible proof generated which can be verified on-chain.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release --bin decompression-game-sp1-evm -- blob_14133451_14136885 --system groth16
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release --bin decompression-game-sp1-evm -- blob_14133451_14136885 --system plonk
//! ```

use clap::{Parser, ValueEnum};
use fuel_proving_games_sp1::decompression_game::evm::{
    create_solidity_context, prove_fixture_groth16, prove_fixture_plonk,
};
use fuel_zkvm_primitives_test_fixtures::decompression_fixtures::Fixture;

/// The arguments for the EVM command.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct EVMArgs {
    #[arg(value_enum)]
    fixture: Fixture,
    #[clap(long, value_enum, default_value = "groth16")]
    system: ProofSystem,
}

/// Enum representing the available proof systems
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum ProofSystem {
    Plonk,
    Groth16,
}

#[tokio::main]
async fn main() -> fuel_proving_games_sp1::Result<()> {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    // Parse the command line arguments.
    let args = EVMArgs::parse();

    let proof_system_name = format!("{:?}", args.system);
    tracing::info!("Proof System: {}", &proof_system_name);

    // Generate the proof based on the selected proof system.
    let (proof, vk) = match args.system {
        ProofSystem::Plonk => prove_fixture_plonk(&args.fixture),
        ProofSystem::Groth16 => prove_fixture_groth16(&args.fixture),
    }?;

    create_solidity_context(&proof, &vk, proof_system_name);

    Ok(())
}
