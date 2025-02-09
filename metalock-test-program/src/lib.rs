
// solana program deploy -u localhost ../target/deploy/metalock_test_program.so


use metalock::{compile::OpEval, prelude::*};
use metalock_core::vm::expr::ToRR;
use solana_program::{compute_units::sol_remaining_compute_units, log::sol_log};

use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    msg,
};




// declare and export the program's entrypoint
entrypoint!(process_instruction);



 
// program entrypoint's implementation
pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    instruction_data: &[u8]
) -> ProgramResult {
    let c1 = sol_remaining_compute_units();
    let c2 = sol_remaining_compute_units();
    let overhead = c1 - c2;
    let instruction_data = &instruction_data[8..];
    let (code, input): (Buffer, RD) = Decode::rd_decode(&mut &*instruction_data).unwrap();
    let mut eval = Evaluator::new(&mut code.as_ref(), Default::default());
    let c1 = sol_remaining_compute_units();
    let r = eval.run(input);
    let c2 = sol_remaining_compute_units();
    msg!("r is: {:?}, cu is: {}", r, c1 - c2 - overhead);
    Ok(())
}
