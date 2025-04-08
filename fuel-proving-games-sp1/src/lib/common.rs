use crate::Error;
use sp1_sdk::{EnvProver, ExecutionReport, SP1ProofWithPublicValues, SP1Stdin, SP1VerifyingKey};
use std::fmt::Debug;

/// Trait for defining game-specific behavior and constants for SP1 games
pub trait GameConfig: Debug + Clone {
    /// Name of the game
    const NAME: &'static str;

    /// The type of fixture used in this game
    type Fixture: Clone + Debug;

    /// Solidity context for the game
    type SolidityContext: Clone + Debug + serde::Serialize;

    /// Get the SP1 ELF for this game
    fn elf() -> &'static [u8];

    /// Get raw input for a specific fixture
    fn get_fixture_input(fixture: &Self::Fixture) -> Vec<u8>;

    /// Get the Solidity context for the game
    fn get_solidity_context(
        proof: &SP1ProofWithPublicValues,
        vk: &SP1VerifyingKey,
    ) -> Self::SolidityContext;
}

#[derive(Debug, Default, Clone, Copy)]
pub enum ProvingMode {
    Plonk,
    Groth16,
    #[default]
    Core,
}

/// A generic prover for SP1 games
#[derive(Debug)]
pub struct GameProver<P, G> {
    prover: P,
    _game: std::marker::PhantomData<G>,
}

impl<P, G> GameProver<P, G>
where
    P: AsRef<EnvProver>,
    G: GameConfig,
{
    /// Create a new GameProver wrapping the given SP1 prover
    pub fn new(prover: P) -> Self {
        Self {
            prover,
            _game: std::marker::PhantomData,
        }
    }

    /// Prove using raw input bytes
    pub fn prove(
        &self,
        input: &[u8],
        mode: ProvingMode,
    ) -> crate::Result<(SP1ProofWithPublicValues, SP1VerifyingKey)> {
        let mut stdin = SP1Stdin::new();
        stdin.write_slice(input);

        // Setup the program for proving
        let (pk, vk) = self.prover.as_ref().setup(G::elf());

        // Generate the proof
        let proof = {
            let prover = self.prover.as_ref().prove(&pk, &stdin);
            let configured_prover = match mode {
                ProvingMode::Core => prover,
                ProvingMode::Groth16 => prover.groth16(),
                ProvingMode::Plonk => prover.plonk(),
            };
            configured_prover
                .run()
                .map_err(|e| Error::FailedToProveProvingGame(e.to_string()))?
        };

        // Return the proof and verification key
        Ok((proof, vk))
    }

    /// Prove a fixture
    pub fn prove_fixture(
        &self,
        fixture: G::Fixture,
        mode: ProvingMode,
    ) -> crate::Result<(SP1ProofWithPublicValues, SP1VerifyingKey)> {
        let raw_input = G::get_fixture_input(&fixture);
        self.prove(&raw_input, mode)
    }

    /// Verify a proof against its verification key
    pub fn verify(
        &self,
        proof: &SP1ProofWithPublicValues,
        vk: &SP1VerifyingKey,
    ) -> crate::Result<()> {
        self.prover
            .as_ref()
            .verify(proof, vk)
            .map_err(|e| Error::FailedToVerifyProof(e.to_string()))
    }

    /// Write the solidity contract fixture to a file
    pub fn create_solidity_fixture(
        &self,
        proof: &SP1ProofWithPublicValues,
        vk: &SP1VerifyingKey,
        path: &str,
    ) -> crate::Result<()> {
        let fixture = G::get_solidity_context(proof, vk);
        let fixture_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path);
        std::fs::create_dir_all(&fixture_path)
            .map_err(|e| Error::FailedToCreateSolidityFixture(anyhow::anyhow!(e)))?;
        std::fs::write(
            fixture_path.join(format!("{}-fixture.json", G::NAME).to_lowercase()),
            serde_json::to_string_pretty(&fixture).unwrap(),
        )
        .expect("failed to write fixture");

        Ok(())
    }
}

/// A generic executor for SP1 games
#[derive(Debug)]
pub struct GameExecutor<E, G> {
    executor: E,
    _game: std::marker::PhantomData<G>,
}

impl<E, G> GameExecutor<E, G>
where
    E: AsRef<EnvProver>,
    G: GameConfig,
{
    /// Create a new GameExecutor wrapping the given SP1 executor
    pub fn new(executor: E) -> Self {
        Self {
            executor,
            _game: std::marker::PhantomData,
        }
    }

    /// Execute with raw input bytes
    pub fn execute(&self, input: &[u8]) -> crate::Result<ExecutionReport> {
        let mut stdin = SP1Stdin::new();
        stdin.write_slice(input);

        // Execute the program
        let (_, report) = self
            .executor
            .as_ref()
            .execute(G::elf(), &stdin)
            .run()
            .map_err(|e| Error::FailedToExecuteProvingGame(e.to_string()))?;

        Ok(report)
    }

    /// Execute a fixture
    pub fn execute_fixture(&self, fixture: G::Fixture) -> crate::Result<ExecutionReport> {
        let raw_input = G::get_fixture_input(&fixture);
        self.execute(&raw_input)
    }
}

/// Helper function to create CSV writer for reports
#[cfg(test)]
pub fn create_csv_writer(
    file_path_env_var: &str,
    default_path: &str,
) -> csv::Writer<std::fs::File> {
    let file_path = std::env::var(file_path_env_var).unwrap_or(default_path.to_string());
    csv::WriterBuilder::new()
        .flexible(true)
        .from_path(file_path)
        .expect("Couldn't create CSV writer")
}
