pub mod core;
pub mod evm;

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const FUEL_SP1_ELF: &[u8] = sp1_sdk::include_elf!("fuel-block-execution-game-sp1");
