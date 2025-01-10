use eggmock::egg::{EGraph, Id, RecExpr};
use eggmock::{Mig, MigLanguage, Network, RewriteFFI, Rewriter, RewriterResult};

struct ExampleRewriter(bool);

impl Rewriter for ExampleRewriter {
    type Network = Mig;
    type Analysis = ();

    fn create_analysis(&mut self) -> Self::Analysis {
        ()
    }

    fn rewrite(
        &mut self,
        _egraph: EGraph<<Self::Network as Network>::Language, Self::Analysis>,
        _roots: impl Iterator<Item = Id>,
    ) -> RewriterResult<Mig> {
        RewriterResult {
            expr: RecExpr::from(vec!(MigLanguage::Const(self.0), MigLanguage::Not(Id::from(0)))),
            roots: vec!(Id::from(0))
        }
    }
}

#[no_mangle]
extern "C" fn example_mig_rewrite() -> RewriteFFI<Mig> {
    RewriteFFI::new(ExampleRewriter(true))
}
