/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const FUEL_SP1_ELF: &[u8] = sp1_sdk::include_elf!("fuel-block-execution-game-sp1");

use crate::common::{GameConfig, GameExecutor, GameProver};
use crate::Result;
use alloy_sol_types::SolType;
use fuel_zkvm_primitives_prover::games::block_execution_game::PublicValuesStruct;
use fuel_zkvm_primitives_test_fixtures::block_execution_fixtures::fixtures::Fixture;
use sp1_sdk::{EnvProver, ExecutionReport, HashableKey, SP1ProofWithPublicValues, SP1VerifyingKey};

/// Configuration for the Block Execution Game
#[derive(Debug, Clone)]
pub struct BlockExecutionGame;

/// A fixture that can be used to test the verification of SP1 zkVM proofs inside Solidity.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolidityContext {
    block_id: [u8; 32],
    input_hash: [u8; 32],
    vkey: String,
    public_values: String,
    proof: String,
}

impl GameConfig for BlockExecutionGame {
    const NAME: &'static str = "block_execution";

    type Fixture = Fixture;

    type SolidityContext = SolidityContext;

    fn elf() -> &'static [u8] {
        FUEL_SP1_ELF
    }

    fn get_fixture_input(fixture: &Self::Fixture) -> Vec<u8> {
        Fixture::get_input_for_fixture(fixture)
    }

    fn get_solidity_context(
        proof: &SP1ProofWithPublicValues,
        vk: &SP1VerifyingKey,
    ) -> Self::SolidityContext {
        let bytes = proof.public_values.as_slice();
        let PublicValuesStruct {
            input_hash,
            block_id,
        } = PublicValuesStruct::abi_decode(bytes, false).unwrap();

        // Create the context so we can test things end-to-end.
        let ctx = SolidityContext {
            block_id: block_id.to_be_bytes(),
            input_hash: input_hash.to_be_bytes(),
            vkey: vk.bytes32().to_string(),
            public_values: format!("0x{}", hex::encode(bytes)),
            proof: format!("0x{}", hex::encode(proof.bytes())),
        };

        ctx
    }
}

/// Type alias for Block Execution Game Prover
pub type BlockExecutionProver<P> = GameProver<P, BlockExecutionGame>;

/// Type alias for Block Execution Game Executor
pub type BlockExecutionExecutor<E> = GameExecutor<E, BlockExecutionGame>;

/// Convenience functions for working with the default prover and executor
pub mod defaults {
    use super::*;
    use std::rc::Rc;

    /// Get a BlockExecutionProver with the default SP1 prover
    pub fn game_prover() -> BlockExecutionProver<Rc<EnvProver>> {
        BlockExecutionProver::new(Rc::new(sp1_sdk::ProverClient::from_env()))
    }

    /// Get a BlockExecutionExecutor with the default SP1 executor
    pub fn game_executor() -> BlockExecutionExecutor<Rc<EnvProver>> {
        BlockExecutionExecutor::new(Rc::new(sp1_sdk::ProverClient::from_env()))
    }

    /// Prove a fixture with the default prover
    pub fn prove_fixture(fixture: Fixture) -> Result<(SP1ProofWithPublicValues, SP1VerifyingKey)> {
        game_prover().prove_fixture(fixture, Default::default())
    }

    /// Execute a fixture with the default executor
    pub fn execute_fixture(fixture: Fixture) -> Result<ExecutionReport> {
        game_executor().execute_fixture(fixture)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::create_csv_writer;
    use fuel_zkvm_primitives_test_fixtures::block_execution_fixtures::fixtures::all_fixtures;
    use serde::Serialize;

    #[derive(Serialize)]
    struct ExecutionReport {
        fixture: Fixture,
        cycle_count: u64,
        memory_address_count: u64,
        syscall_count: u64,
    }

    #[derive(Serialize)]
    struct ProvingReport {
        fixture: Fixture,
        proving_time: u128,
        verification_time: u128,
    }

    #[test]
    fn run_all_fixtures_and_collect_report() {
        let fixtures = all_fixtures();
        let mut wtr = create_csv_writer("FUEL_SP1_REPORT", "fuel_sp1_block_execution_report.csv");

        // Create a reusable executor
        let executor = defaults::game_executor();

        for fixture in fixtures {
            // Execute the fixture
            let report = executor.execute_fixture(fixture.clone()).unwrap();

            let perf_report = ExecutionReport {
                fixture: fixture.clone(),
                cycle_count: report.total_instruction_count(),
                memory_address_count: report.touched_memory_addresses,
                syscall_count: report.total_syscall_count(),
            };

            wtr.serialize(perf_report).expect("Couldn't write to CSV");
            wtr.flush().expect("Couldn't flush CSV writer");

            tracing::info!("Executed fixture: {:?}", fixture);
        }
    }

    #[test]
    fn prove_all_fixtures_and_collect_report() {
        let fixtures = all_fixtures();
        let mut wtr = create_csv_writer("FUEL_SP1_REPORT", "fuel_sp1_block_execution_report.csv");

        // Create a reusable prover
        let prover = defaults::game_prover();

        for fixture in fixtures {
            // Prove the fixture
            let start_time = std::time::Instant::now();
            let (proof, vk) = prover
                .prove_fixture(fixture.clone(), Default::default())
                .unwrap();
            let proving_time = start_time.elapsed().as_millis();

            let start_time = std::time::Instant::now();
            prover.verify(&proof, &vk).expect("failed to verify proof");
            let verification_time = start_time.elapsed().as_millis();

            let perf_report = ProvingReport {
                fixture: fixture.clone(),
                proving_time,
                verification_time,
            };

            wtr.serialize(perf_report).expect("Couldn't write to CSV");
            wtr.flush().expect("Couldn't flush CSV writer");

            tracing::info!("Proved fixture: {:?}", fixture);
        }
    }
}
