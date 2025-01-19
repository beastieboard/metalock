

use anchor_lang::prelude::*;

use super::Buffer;


#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MetalockProxyCall {
    pub program_id: Pubkey,
    pub data: Buffer,
    pub accounts: Vec<AccountMeta>
}
