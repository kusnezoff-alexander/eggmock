use std::ops::Index;
use egg::{Analysis, CostFunction, EGraph, Extractor, Language};
use crate::{Id, NetworkLanguage, Network, Receiver, Signal};

fn egraph_id_for_signal<L: NetworkLanguage, A: Analysis<L>>(
    graph: &mut EGraph<L, A>,
    signal: Signal,
) -> egg::Id {
    let child_id = signal.node_id().into();
    if signal.is_inverted() {
        graph.add(L::not(child_id))
    } else {
        child_id
    }
}

impl<L: NetworkLanguage, A: Analysis<L>> Receiver for EGraph<L, A> {
    type Node = L::Node;
    type Result = (Self, Vec<egg::Id>);

    fn create_node(&mut self, node: Self::Node) -> Signal {
        let node = L::from_node(node, |signal| egraph_id_for_signal(self, signal));
        Signal::new(Id::from(self.add(node)), false)
    }

    fn done(mut self, outputs: &[Signal]) -> Self::Result {
        let outputs = Vec::from_iter(
            outputs
                .iter()
                .map(|signal| egraph_id_for_signal(&mut self, *signal)),
        );
        (self, outputs)
    }
}

impl<L: NetworkLanguage, CF: CostFunction<L>, A: Analysis<L>> Network
for (Extractor<'_, CF, L, A>, Vec<egg::Id>)
{
    type Node = L::Node;

    fn outputs(&self) -> impl Iterator<Item = Signal> {
        self.1
            .iter()
            .map(|o| ExtractorIndexWrapper(&self.0).to_signal(*o))
    }

    fn node(&self, id: Id) -> Self::Node {
        self.0
            .find_best_node(id.into())
            .to_node(|id| ExtractorIndexWrapper(&self.0).to_signal(id))
            .expect("id should point to a non-not node")
    }
}

pub trait EggIdToSignal {
    fn to_signal(&self, id: egg::Id) -> Signal;
}

impl<I: Index<egg::Id, Output: NetworkLanguage>> EggIdToSignal for I {
    fn to_signal(&self, mut id: egg::Id) -> Signal {
        let mut invert = false;
        loop {
            let node = &self[id];
            if node.is_not() {
                invert = !invert;
                id = node.children()[0];
            } else {
                break Signal::new(id.into(), invert);
            }
        }
    }
}

struct ExtractorIndexWrapper<'r, E>(&'r E);

impl<CF, L, A> Index<egg::Id> for ExtractorIndexWrapper<'_, Extractor<'_, CF, L, A>>
where
    CF: CostFunction<L>,
    L: NetworkLanguage,
    A: Analysis<L>,
{
    type Output = L;

    fn index(&self, index: egg::Id) -> &Self::Output {
        self.0.find_best_node(index)
    }
}
