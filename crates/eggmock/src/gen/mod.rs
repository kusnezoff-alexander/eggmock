mod rewrite;
mod transfer;

use crate::{Network, RewriteCallback, RewriteFFI, TransferFFI};
use indoc::formatdoc;

pub trait CFFI {
    fn c_ffi() -> String;
}

trait StructName {
    fn struct_name() -> String;
}

pub fn network_ffi<N>() -> String
where
    N: Network,
    N::TransferFFI: CFFI,
{
    formatdoc!(
        "
            {}
            {}
            {}
        ",
        N::TransferFFI::c_ffi(),
        RewriteCallback::<N>::c_ffi(),
        RewriteFFI::<N>::c_ffi(),
    )
}
