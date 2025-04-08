use crate::{decompression_game::FUEL_SP1_ELF, Error, Result};

use alloy_sol_types::SolType;
use fuel_zkvm_primitives_prover::games::decompression_game::{Input, PublicValuesStruct};
use fuel_zkvm_primitives_test_fixtures::decompression_fixtures::Fixture;
use sp1_sdk::{EnvProver, ExecutionReport, SP1ProofWithPublicValues, SP1Stdin, SP1VerifyingKey};

pub fn add_fixture_input_to_stdin(fixture: Fixture, stdin: &mut SP1Stdin) -> Result<()> {
    let raw_input = Fixture::get_input_for_fixture(&fixture);
    let input: Input = bincode::deserialize(&raw_input).map_err(Error::FailedToDeserializeInput)?;
    stdin.write(&input);

    Ok(())
}

pub fn execute_fixture(
    fixture: Fixture,
    client: &EnvProver,
    mut stdin: SP1Stdin,
) -> Result<ExecutionReport> {
    add_fixture_input_to_stdin(fixture, &mut stdin)?;

    // Execute the program
    let (output, report) = client
        .execute(FUEL_SP1_ELF, &stdin)
        .run()
        .map_err(|e| Error::FailedToExecuteProvingGame(e.to_string()))?;
    tracing::info!("Program executed successfully.");

    let output = PublicValuesStruct::abi_decode(output.as_slice(), true)
        .map_err(|e| Error::FailedToDeserializePublicOutput(e.to_string()))?;

    if output.first_block_height > output.last_block_height {
        Err(Error::Fault(format!(
            "First block height is greater than last block height: {} > {}",
            output.first_block_height, output.last_block_height
        )))?;
    }

    Ok(report)
}

pub fn prove_fixture(
    fixture: Fixture,
    client: &EnvProver,
    mut stdin: SP1Stdin,
) -> Result<(SP1ProofWithPublicValues, SP1VerifyingKey)> {
    let _ = add_fixture_input_to_stdin(fixture, &mut stdin)?;

    // Setup the program for proving.
    let (pk, vk) = client.setup(FUEL_SP1_ELF);

    // Generate the proof
    let proof = client
        .prove(&pk, &stdin)
        .run()
        .map_err(|e| Error::FailedToProveProvingGame(e.to_string()))?;

    Ok((proof, vk))
}

#[cfg(test)]
mod tests {
    use super::*;
    use csv::Writer;
    use fuel_zkvm_primitives_test_fixtures::decompression_fixtures::all_fixtures;
    use serde::Serialize;
    use sp1_sdk::ProverClient;

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

        let file_path = std::env::var("FUEL_SP1_REPORT")
            .unwrap_or("fuel_sp1_decompression_report.csv".to_string());
        let mut wtr = Writer::from_path(file_path).expect("Couldn't create CSV writer");
        let prover_client = ProverClient::from_env();

        for fixture in fixtures {
            let stdin = SP1Stdin::new();
            let report = execute_fixture(fixture.clone(), &prover_client, stdin).unwrap();

            let perf_report = ExecutionReport {
                fixture: fixture.clone(),
                cycle_count: report.total_instruction_count(),
                memory_address_count: report.touched_memory_addresses,
                syscall_count: report.total_syscall_count(),
            };

            wtr.serialize(perf_report).expect("Couldn't write to CSV");

            // flush after each execution
            wtr.flush().expect("Couldn't flush CSV writer");

            tracing::info!("Executed fixture: {:?}", fixture);
        }
    }

    #[test]
    fn prove_all_fixtures_and_collect_report() {
        let fixtures = all_fixtures();

        let file_path = std::env::var("FUEL_SP1_REPORT")
            .unwrap_or("fuel_sp1_decompression_report.csv".to_string());
        let mut wtr = Writer::from_path(file_path).expect("Couldn't create CSV writer");
        let client = ProverClient::from_env();

        for fixture in fixtures {
            let stdin = SP1Stdin::new();

            let start_time = std::time::Instant::now();
            let (proof, vk) = prove_fixture(fixture.clone(), &client, stdin).unwrap();
            let proving_time = start_time.elapsed().as_millis();

            let start_time = std::time::Instant::now();
            client.verify(&proof, &vk).expect("failed to verify proof");
            let verification_time = start_time.elapsed().as_millis();

            let perf_report =
                ProvingReport { fixture: fixture.clone(), proving_time, verification_time };

            wtr.serialize(perf_report).expect("Couldn't write to CSV");

            // flush after each execution
            wtr.flush().expect("Couldn't flush CSV writer");

            tracing::info!("Proved fixture: {:?}", fixture);
        }
    }
}
