#![cfg(feature = "test-sbf")]

use {
    anchor_lang::{solana_program::instruction::Instruction, InstructionData, ToAccountMetas},
    mollusk_svm::{result::Check, Mollusk},
};

#[test]
fn test_initialize() {
    let program_id = anchor_amm::id();

    let mollusk = Mollusk::new(&program_id, "anchor_amm");

    let instruction = Instruction::new_with_bytes(
        program_id,
        &anchor_amm::instruction::Initialize {}.data(),
        anchor_amm::accounts::Initialize {}.to_account_metas(None),
    );

    mollusk.process_and_validate_instruction(&instruction, &[], &[Check::success()]);
}
