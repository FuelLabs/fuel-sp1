//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can be executed
//! or have a core proof generated.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release --bin decompression-game-sp1 -- execute_fixture blob_14133451_14136885
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release --bin decompression-game-sp1 -- prove_fixture blob_14133451_14136885 core
//! ```

use clap::{Parser, Subcommand};
use fuel_proving_games_sp1::decompression_game::defaults;
use fuel_zkvm_primitives_test_fixtures::decompression_fixtures::Fixture;

/// The arguments for the command.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
#[clap(
    name = "command",
    about = "The command to execute",
    rename_all = "snake_case"
)]
enum Command {
    ExecuteFixture {
        #[arg(value_enum)]
        fixture: Fixture,
    },
    ProveFixture {
        #[arg(value_enum)]
        fixture: Fixture,
        mode: ProvingMode,
        output_path: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum ProvingMode {
    Plonk,
    Groth16,
    Core,
}

impl From<ProvingMode> for fuel_proving_games_sp1::common::ProvingMode {
    fn from(value: ProvingMode) -> Self {
        match value {
            ProvingMode::Plonk => Self::Plonk,
            ProvingMode::Groth16 => Self::Groth16,
            ProvingMode::Core => Self::Core,
        }
    }
}

fn main() -> fuel_proving_games_sp1::Result<()> {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    // Parse the command line arguments.
    let args = Args::parse();

    match args.command {
        Command::ExecuteFixture { fixture } => {
            tracing::info!("Executing the fixture.");

            // Execute the program using the default executor
            let report = defaults::execute_fixture(fixture)?;
            tracing::info!("fixture executed successfully.");

            // Record the number of cycles executed.
            tracing::info!("Number of cycles: {}", report.total_instruction_count());
        }
        Command::ProveFixture {
            fixture,
            mode,
            output_path,
        } => {
            tracing::info!("Proving and verifying the fixture.");

            // Get the default prover
            let prover = defaults::game_prover();

            // Generate the proof
            let (proof, vk) = prover.prove_fixture(fixture, mode.into())?;

            // Verify the proof
            prover.verify(&proof, &vk).expect("failed to verify proof");
            tracing::info!("Successfully generated and verified proof!");

            match mode {
                ProvingMode::Plonk | ProvingMode::Groth16 => prover.create_solidity_fixture(
                    &proof,
                    &vk,
                    &output_path.unwrap_or("contracts/".into()),
                )?,
                _ => {}
            }
        }
    }

    Ok(())
}
