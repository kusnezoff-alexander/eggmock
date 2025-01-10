use super::StructFFI;
use crate::{Network, RewriteCallback, RewriteFFI};
use indoc::formatdoc;

impl<N> StructFFI for RewriteFFI<N>
where
    N: Network,
    N::TransferFFI: StructFFI,
{
    fn struct_name() -> String {
        format!("{}_rewrite", N::TYPENAME)
    }
    fn struct_definition() -> String {
        formatdoc!(
            r#"
            struct {}
            {{
              void* data;
              {} transfer;
              void ( *rewrite )( void* data, uint64_t* roots, size_t roots_size, {} callback );
              void ( *free )( void* data );
            }};
            "#,
            Self::struct_name(),
            N::TransferFFI::struct_name(),
            RewriteCallback::<N>::struct_name()
        )
    }
}

impl<N> StructFFI for RewriteCallback<N>
where
    N: Network,
    N::TransferFFI: StructFFI,
{
    fn struct_name() -> String {
        format!("{}_rewrite_callback", N::TYPENAME)
    }
    fn struct_definition() -> String {
        formatdoc!(
            r#"
            struct {}
            {{
              void* data;
              {} transfer;
            }};
            "#,
            Self::struct_name(),
            N::TransferFFI::struct_name()
        )
    }
}

pub fn rewrite_helper<N: Network>() -> String {
    let rewrite_struct = RewriteFFI::<N>::struct_name();
    let ntk = N::TYPENAME;
    let ntk_type = format!("mockturtle::{}", N::MOCKTURTLE_TYPENAME);
    formatdoc!(
        r#"
        inline {ntk_type} rewrite_{ntk}( {ntk_type} const& in_ntk, {rewrite_struct} const& rewrite ) {{
          transfer_{ntk} ( in_ntk, rewrite.data, rewrite.transfer );

          std::vector<uint64_t> roots;
          roots.reserve( in_ntk.num_pos() );
          for ( uint32_t i = 0; i < in_ntk.num_pos(); i++ )
          {{
            roots.emplace_back( in_ntk.po_at( i ).data );
          }}

          {ntk_type} out_ntk;
          {ntk}_rewrite_callback callback = {{
              .data = &out_ntk,
              .transfer = receive_{ntk}(),
          }};

          rewrite.rewrite( rewrite.data, roots.data(), roots.size(), callback );

          for ( auto root : roots )
          {{
            auto signal = {ntk_type}::signal( root );
            out_ntk.create_po( signal );
          }}

          rewrite.free( rewrite.data );
          return out_ntk;
        }}
        "#)
}
