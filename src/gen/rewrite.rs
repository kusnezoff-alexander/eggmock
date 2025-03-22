use crate::Node;
use indoc::formatdoc;

pub fn rewrite_struct<N: Node>() -> String {
    let ntk = N::NTK_TYPENAME;
    formatdoc!(
        r#"
        struct {ntk}_rewrite
        {{
          void* data;
          void ( *rewrite )( void* data, {ntk}_receiver<void> callback );
        }};
        "#,
    )
}

pub fn rewrite_helper<N: Node>() -> String {
    let ntk = N::NTK_TYPENAME;
    let ntk_type = format!("mockturtle::{}", N::NTK_MOCKTURTLE_TYPENAME);
    formatdoc!(
        r#"
        inline {ntk_type} rewrite_{ntk}( {ntk_type} const& in_ntk, {ntk}_receiver<{ntk}_rewrite> const& receiver )
        {{
          auto rewrite = send_{ntk}( in_ntk, receiver );
          {ntk_type} out_ntk;
          auto callback = receive_{ntk}( out_ntk );
          rewrite.rewrite( rewrite.data, callback );
          return out_ntk;
        }}
        "#
    )
}
