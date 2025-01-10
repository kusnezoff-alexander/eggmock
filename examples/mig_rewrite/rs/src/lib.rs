use std::cmp::Ordering;
use std::collections::HashSet;
use std::rc::Rc;
use eggmock::egg::{rewrite, CostFunction, EGraph, Id, Language, Runner};
use eggmock::{Mig, MigLanguage, Network, RewriteFFI, Rewriter, RewriterResult};
use std::time::Duration;

/// Optimizes for number of constant-users
struct ExampleCostFunction;

#[derive(Debug, Clone)]
struct ExampleCost(Rc<HashSet<MigLanguage>>);

impl PartialEq<Self> for ExampleCost {
    fn eq(&self, other: &Self) -> bool {
        self.0.len() == other.0.len()
    }
}

impl PartialOrd for ExampleCost {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.len().partial_cmp(&other.0.len())
    }
}

impl CostFunction<MigLanguage> for ExampleCostFunction {
    type Cost = ExampleCost;

    fn cost<C>(&mut self, enode: &MigLanguage, mut costs: C) -> Self::Cost
    where
        C: FnMut(Id) -> Self::Cost,
    {
        if let &MigLanguage::Const(_) = enode {
            return ExampleCost(Rc::new(HashSet::from([enode.clone()])));
        }
        let mut set = HashSet::new();
        for child in enode.children() {
            for r in costs(*child).0.iter() {
                if let MigLanguage::Const(_) = r {
                    set.insert(enode.clone());
                } else {
                    set.insert(r.clone());
                }
            }
        }
        ExampleCost(Rc::new(set))
    }
}

struct ExampleRewriter;

impl Rewriter for ExampleRewriter {
    type Network = Mig;
    type Analysis = ();

    fn create_analysis(&mut self) -> Self::Analysis {
        ()
    }

    fn rewrite(
        &mut self,
        graph: EGraph<<Self::Network as Network>::Language, Self::Analysis>,
        roots: impl Iterator<Item = Id>,
    ) -> RewriterResult<Mig> {
        let rules = &[
            rewrite!("commute_1"; "(maj ?a ?b ?c)" => "(maj ?b ?a ?c)"),
            rewrite!("commute_2"; "(maj ?a ?b ?c)" => "(maj ?a ?c ?b)"),
            rewrite!("not_true"; "(! true)" => "false"),
            rewrite!("not_false"; "(! false)" => "true" ),
            rewrite!("example"; "(maj true ?a (maj false ?b ?c))" => "(maj true ?a (maj ?a ?b ?c))"),
        ];
        let runner = Runner::default()
            .with_time_limit(Duration::from_secs(60))
            .with_egraph(graph)
            .run(rules);
        runner.print_report();
        RewriterResult::extract_greedy(&runner.egraph, ExampleCostFunction, roots)
    }
}

#[no_mangle]
extern "C" fn example_mig_rewrite() -> RewriteFFI<Mig> {
    RewriteFFI::new(ExampleRewriter)
}
