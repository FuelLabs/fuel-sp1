use crate::{block_execution_game::FUEL_SP1_ELF, Error, Result};
use alloy_sol_types::SolType;
use fuel_zkvm_primitives_prover::games::block_execution_game::{Input, PublicValuesStruct};
use fuel_zkvm_primitives_test_fixtures::block_execution_fixtures::fixtures::Fixture;
use serde::{Deserialize, Serialize};
use sp1_sdk::{
    EnvProver, HashableKey, ProverClient, SP1ProofWithPublicValues, SP1ProvingKey, SP1Stdin,
    SP1VerifyingKey,
};
use std::path::PathBuf;

struct EvmProvingContext {
    client: EnvProver,
    stdin: SP1Stdin,
    pk: SP1ProvingKey,
    vk: SP1VerifyingKey,
}

fn setup_fixture(fixture: &Fixture) -> Result<EvmProvingContext> {
    // Setup the prover client.
    let client = ProverClient::from_env();

    // Setup the program.
    let (pk, vk) = client.setup(FUEL_SP1_ELF);

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();

    let raw_input = Fixture::get_input_for_fixture(fixture);
    let input: Input = bincode::deserialize(&raw_input).map_err(Error::FailedToDeserializeInput)?;
    stdin.write(&input);

    let proving_context = EvmProvingContext { client, stdin, pk, vk };

    Ok(proving_context)
}

pub fn prove_fixture_plonk(
    fixture: &Fixture,
) -> Result<(SP1ProofWithPublicValues, SP1VerifyingKey)> {
    let EvmProvingContext { client, pk, stdin, vk } = setup_fixture(fixture)?;

    // Prove the fixture.
    let proof = client
        .prove(&pk, &stdin)
        .plonk()
        .run()
        .map_err(|e| Error::FailedToProveProvingGame(e.to_string()))?;

    Ok((proof, vk))
}

pub fn prove_fixture_groth16(
    fixture: &Fixture,
) -> Result<(SP1ProofWithPublicValues, SP1VerifyingKey)> {
    let EvmProvingContext { client, pk, stdin, vk } = setup_fixture(fixture)?;

    // Prove the fixture.
    let proof = client
        .prove(&pk, &stdin)
        .groth16()
        .run()
        .map_err(|e| Error::FailedToProveProvingGame(e.to_string()))?;

    Ok((proof, vk))
}

/// A fixture that can be used to test the verification of SP1 zkVM proofs inside Solidity.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SolidityContext {
    block_id: [u8; 32],
    input_hash: [u8; 32],
    vkey: String,
    public_values: String,
    proof: String,
}

/// Create a solidity contract + test for the given proof and verification key
pub fn create_solidity_context(
    proof: &SP1ProofWithPublicValues,
    vk: &SP1VerifyingKey,
    system: String,
) {
    // Deserialize the public values.
    let bytes = proof.public_values.as_slice();
    let PublicValuesStruct { input_hash, block_id } =
        PublicValuesStruct::abi_decode(bytes, false).unwrap();

    // Create the testing fixture so we can test things end-to-end.
    let fixture = SolidityContext {
        block_id: block_id.to_be_bytes(),
        input_hash: input_hash.to_be_bytes(),
        vkey: vk.bytes32().to_string(),
        public_values: format!("0x{}", hex::encode(bytes)),
        proof: format!("0x{}", hex::encode(proof.bytes())),
    };

    // The verification key is used to verify that the proof corresponds to the execution of the
    // program on the given input.
    //
    // Note that the verification key stays the same regardless of the input.
    tracing::info!("Verification Key: {}", fixture.vkey);

    // The public values are the values which are publicly committed to by the zkVM.
    //
    // If you need to expose the inputs or outputs of your program, you should commit them in
    // the public values.
    tracing::info!("Public Values: {}", fixture.public_values);

    // The proof proves to the verifier that the program was executed with some inputs that led to
    // the give public values.
    tracing::info!("Proof Bytes: {}", fixture.proof);

    // Save the fixture to a file.
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../contracts/src/fixtures");
    std::fs::create_dir_all(&fixture_path).expect("failed to create fixture path");
    std::fs::write(
        fixture_path.join(format!("{:?}-fixture.json", system).to_lowercase()),
        serde_json::to_string_pretty(&fixture).unwrap(),
    )
    .expect("failed to write fixture");
}
