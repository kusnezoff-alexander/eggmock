use rustc_hash::FxHashMap;
use crate::{Id, Node, Provider, Receiver, Signal};

pub trait ProviderWithBackwardEdges: Provider {
    fn outputs(&self, id: Id) -> impl Iterator<Item = Id> + '_;
    fn leafs(&self) -> impl Iterator<Item = Id> + '_;
}

pub struct ComputedProviderWithBackwardEdges<'a, P: ?Sized> {
    provider: &'a P,
    backward: FxHashMap<Id, Vec<Id>>,
    leafs: Vec<Id>,
}

impl<'a, P: Provider + ?Sized> ComputedProviderWithBackwardEdges<'a, P> {
    pub fn new(provider: &'a P) -> Self {
        let mut backward = FxHashMap::default();
        let mut leafs = Vec::new();
        for (output_id, output) in provider.iter() {
            let inputs = output.inputs();
            for i in 0..inputs.len() {
                let input_signal = inputs[i];
                let input_id = input_signal.node_id();
                // prevent duplicate entries in the Vecs
                if inputs[0..i].iter().map(Signal::node_id).any(|id| id == input_id) {
                    continue;
                }
                backward.entry(input_id).or_insert_with(Vec::new).push(output_id);
            }
            if inputs.len() == 0 {
                leafs.push(output_id);
            }
        }
        Self { provider, backward, leafs }
    }
}

impl<P: Provider + ?Sized> Provider for ComputedProviderWithBackwardEdges<'_, P> {
    type Node = P::Node;

    fn outputs(&self) -> impl Iterator<Item=Signal> {
        self.provider.outputs()
    }
    fn node(&self, id: Id) -> Self::Node {
        self.provider.node(id)
    }
    fn iter(&self) -> impl Iterator<Item=(Id, Self::Node)> + '_ {
        self.provider.iter()
    }
    fn send<R: Receiver<Node=Self::Node>>(&self, receiver: R) -> R::Result {
        self.provider.send(receiver)
    }
}

impl<P: Provider + ?Sized> ProviderWithBackwardEdges for ComputedProviderWithBackwardEdges<'_, P> {
    fn outputs(&self, id: Id) -> impl Iterator<Item=Id> + '_ {
        self.backward.get(&id).into_iter().flat_map(|v| v.iter()).cloned()
    }
    fn leafs(&self) -> impl Iterator<Item=Id> + '_ {
        self.leafs.iter().cloned()
    }
}
