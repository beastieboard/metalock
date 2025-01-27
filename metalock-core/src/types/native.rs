
use solana_program::{instruction::AccountMeta, pubkey::Pubkey};

macro_rules! offset_of {
    ($struct:ty, $field:ident) => {{
        let dummy_ptr = std::ptr::null::<$struct>();
        let field_ptr = unsafe { &(*dummy_ptr).$field as *const _ };
        (field_ptr as usize) - (dummy_ptr as usize)
    }};
}

use super::core::*;
use super::tags::*;
use super::data::*;
pub use super::data::Native;
use super::schema::*;
use super::encode::*;

use std::marker::PhantomData;
use crate::vm::expr::{ToRR, RR, GetStructField, SetStructField};
use paste::paste;




pub trait NativeData: SchemaType {}




macro_rules! rr_native_struct {
    ($struct:ident { $( $idx:literal $field:ident: $type:ty ),* }) => {
        paste! {
            impl RR<$struct> {
            $(  pub fn [<get_ $field>](&self) -> RR<$type> {
                    let offset = offset_of!($struct, $field);
                    RR::new(GetStructField(self.clone(), $idx, offset as u32, PhantomData::default()))
                }
                pub fn [<set_ $field>](&self, val: impl ToRR<$type>) -> Self {
                    let offset = offset_of!($struct, $field);
                    RR::new(SetStructField(self.clone(), $idx, offset as u32, val.rr()))
                } )*
            }
        }
        impl SchemaType for $struct {
            //type Items = tlist!(tag::RSTRUCT, u16, $($type),*);
            fn encode_schema(out: &mut Vec<u8>) {
                out.push(tag::RSTRUCT::ID);
                out.extend((std::mem::size_of::<Self>() as u16).rd_encode());
                out.push(0$(.max($idx+1))*);
                $(<$type>::encode_schema(out);)*
            }
        }
        impl NativeData for $struct {}
        impl Into<RD> for $struct {
            fn into(self) -> RD {
                Native::from(self).into()
            }
        }
    };
}


rr_native_struct!(
    AccountMeta {
        0 pubkey: Pubkey,
        1 is_signer: bool,
        2 is_writable: bool
    }
);


#[derive(Clone, Debug)]
pub struct MetalockProxyCall {
    pub program_id: Pubkey,
    pub data: Buffer,
    pub accounts: Vec<AccountMeta>
}

rr_native_struct!(
    MetalockProxyCall {
        0 program_id: Pubkey,
        1 data: Buffer,
        2 accounts: Vec<AccountMeta>
    }
);

