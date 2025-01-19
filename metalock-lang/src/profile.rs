




pub const SOLANA_MEMORY_START: usize = 0x300000000;
pub const MEMORY_RESERVED: usize = 0x100;


#[derive(Debug)]
#[repr(usize)]
pub(crate) enum ProfileLabel {
    IntoRdNative,
    IntoRdVec,
    IntoRdOption,
    IntoRdU64,
    IntoRd128,
    IntoRdString,
    IntoRdPubkey,
    IntoRdBuffer,
    RdCreatePtr,
    IntoRd,
}

static LAST_LABEL: usize = unsafe { std::mem::transmute(ProfileLabel::IntoRd) };


pub fn profile_dump(_len: usize) {
    #[cfg(all(feature = "metalock-profile", not(feature = "no-profile")))]
    unsafe {
        use anchor_lang::prelude::msg;
        let mut items = (0..LAST_LABEL+1).map(|i| {
            let label: ProfileLabel = std::mem::transmute(i);
            let p = ((i*8)+SOLANA_MEMORY_START) as *const u32;
            (format!("{:?}", label), *p, *p.add(1))
        }).collect::<Vec<_>>();
        items.sort_by(|(_, a, _), (_, b, _)| b.cmp(a));
        for (l, cu, n) in items.into_iter().filter(|o| o.2 > 0).take(len) {
            msg!("console.log Profile: {}: {} (n={}, {}/ea)", l, cu, n, cu/n);
        }
    }
}

//pub use anchor_lang::solana_program::entrypoint::BumpAllocator;

#[cfg(all(feature = "metalock-profile", not(feature = "no-profile")))]
mod macros {
    #[macro_export]
    macro_rules! profile_wrap {
        ($code:ident, $body:expr) => {
            {
                let mut cu = anchor_lang::solana_program::compute_units::sol_remaining_compute_units();
                let r = $body;
                {
                    let cu2 = anchor_lang::solana_program::compute_units::sol_remaining_compute_units();
                    cu -= cu2;
                    let off: usize = unsafe { std::mem::transmute(crate::profile::ProfileLabel::$code) };
                    let p = (0x300000000 + off * 8) as *mut u32;
                    unsafe { *p += cu as u32 - 100 };
                    unsafe { *p.add(1) += 1 };
                }
                r
            }
        };
    }
    #[macro_export]
    macro_rules! metalock_profile_allocator {
        () => {
            const SOLANA_MEMORY_START: usize = 0x300000000;
            const MEMORY_RESERVED: usize = 0x100;
            use anchor_lang::solana_program::entrypoint::BumpAllocator;
            #[global_allocator]
            pub static A: BumpAllocator = BumpAllocator {
                start: SOLANA_MEMORY_START + MEMORY_RESERVED,
                len: 32 * 1024 - MEMORY_RESERVED,
            };
        };
    }
}
#[cfg(not(all(feature = "metalock-profile", not(feature = "no-profile"))))]
mod macros {
    #[macro_export]
    macro_rules! profile_wrap {
        ($code:ident, $body:expr) => { $body };
    }
    #[macro_export]
    macro_rules! metalock_profile_allocator { () => { }; }
}




