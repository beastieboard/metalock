

use std::str::FromStr;

use metalock::prelude::*;
use metalock_core::vm::expr::ToRR;
use metalock::compile::OpEval;

use solana_client::rpc_client::RpcClient;
use solana_client::rpc_request::RpcRequest;
use solana_sdk::{
    account_info::AccountInfo, compute_budget::ComputeBudgetInstruction, entrypoint::ProgramResult, instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::{SeedDerivable, Signer}, transaction::Transaction
};


// 4zvwRjXUKGfvwnParsHAS3HuSVzV5cA4McphgmoCtajS
// solana airdrop -u localhost 1 4zvwRjXUKGfvwnParsHAS3HuSVzV5cA4McphgmoCtajS




pub(crate) fn test_program_cu<A: SchemaType + Into<RD>>(code: Vec<u8>, input: A) {

    let input = input.rr().encode()[1..].to_vec();
    let code = Buffer(code.to_vec()).rd_encode();
    let data = [code, input].concat();

    let rpc_url = "http://localhost:8899";
    let client = RpcClient::new(rpc_url);
    let program_id = Pubkey::from_str("25HvKnnRy2pYnDhUQDgJSADTiHkDv8AoN1zDBM3y2pFB").unwrap();
    let your_instruction = Instruction::new_with_bincode(program_id, &data, vec![]);
    let payer = Keypair::from_seed(&[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]).unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[your_instruction],
        Some(&payer.pubkey()),
        &[&payer],
        client.get_latest_blockhash().unwrap()
    );

    let result = client.simulate_transaction(&tx).unwrap();
    println!("{:?}", result.value.logs.unwrap()[1]);
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cu() {
        fn not_10(n: RR<u32>) -> RR<bool> {
            n.equals(10).not()
        }
        let code = not_10.to_program().compile();

        test_program_cu(code, 1u32);
    }

    #[test]
    fn test_match() {
        type T = Result<[u8;7], (u16, u16, u16)>;

        for i in 0u64..255 {
            let t: T = unsafe { std::mem::transmute(i) };
            match t {
                Ok(i) => {
                    println!("Ok: {:?}", i);
                },
                Err(_) => {
                    println!("Err: {:?}", i);
                }
            }
        }
    }
}

