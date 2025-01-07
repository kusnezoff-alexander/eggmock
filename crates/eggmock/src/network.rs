use egg::{Analysis, EGraph, Id, Language, RecExpr};

pub trait Node {
    const TYPENAME: &'static str;

    type Variants: NodeType<Language = Self>;
    const TYPES: &'static [Self::Variants];

    fn c_ffi() -> String {
        let typename = Self::TYPENAME;

        let mut fields = "uint64_t ( *add_symbol )( void* data, uint64_t name );".to_string();
        for variant in Self::TYPES {
            fields += "\n  uint64_t ( *add_";
            fields += variant.name();
            fields += " )( void* data";
            for i in 1..=variant.fanin() {
                fields += ", uint64_t id";
                fields += i.to_string().as_str();
            }
            fields += " );"
        }

        format!(
            r#"
#ifdef __cplusplus
extern "C" {{
#endif

#include <stdint.h>
#include <stddef.h>

struct eggmock_{typename}_ffi;
struct eggmock_{typename}_ffi_callback;

struct eggmock_{typename}_ffi
{{
  void* data;
  {fields}
  void ( *rewrite )( void* data, size_t roots_size, const uint64_t* roots, eggmock_{typename}_ffi_callback callback );
  void ( *free )( void* data );
}};

struct eggmock_{typename}_ffi_callback
{{
  void* data;
  {fields}
  void ( *mark_roots )( void* data, size_t roots_size, const uint64_t* roots );
}};


#ifdef __cplusplus
}}
#endif"#
        )
    }
}

pub trait NodeType: Sized + 'static {
    type Language: Node<Variants = Self>;

    // const MOCKTURTLE_CREATE_METHOD: &'static str;
    // const MOCKTURTLE_IS_METHOD: &'static str;

    fn name(&self) -> &'static str;
    fn fanin(&self) -> u8;
}

pub trait Rewriter: Sized {
    type Network: Node + Language;
    type Analysis: Analysis<Self::Network>;

    fn create_analysis(&mut self) -> Self::Analysis;

    fn rewrite(
        &mut self,
        egraph: EGraph<Self::Network, Self::Analysis>,
        roots: impl Iterator<Item = Id>,
    ) -> RewriterResult<Self>;
}

pub struct RewriterResult<R: Rewriter> {
    pub expr: RecExpr<R::Network>,
    pub roots: Vec<Id>,
}
