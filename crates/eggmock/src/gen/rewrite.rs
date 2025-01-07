use indoc::formatdoc;
use crate::gen::{StructName, CFFI};
use crate::{Network, RewriteCallback, RewriteFFI};

impl<N: Network> StructName for RewriteFFI<N> {
    fn struct_name() -> String {
        format!("eggmock_{}_rewrite", N::TYPENAME)
    }
}

impl<N: Network> StructName for RewriteCallback<N> {
    fn struct_name() -> String {
        format!("eggmock_{}_rewrite_callback", N::TYPENAME)
    }
}

impl<N: Network> CFFI for RewriteFFI<N>
where
    N::TransferFFI: StructName,
{
    fn c_ffi() -> String {
        formatdoc!(
            r#"
                struct {} {{
                    void* data;
                    {} transfer;
                    void ( *rewrite )( void* data, uint64_t* roots, size_t roots_size, {} callback );
                }};
            "#,
            Self::struct_name(),
            N::TransferFFI::struct_name(),
            RewriteCallback::<N>::struct_name()
        )
    }
}

impl<N: Network> CFFI for RewriteCallback<N>
where
    N::TransferFFI: StructName,
{
    fn c_ffi() -> String {
        formatdoc!(
            r#"
                struct {} {{
                    void* data;
                    {} transfer;
                }};
            "#,
            Self::struct_name(),
            N::TransferFFI::struct_name()
        )
    }
}