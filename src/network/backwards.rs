use rustc_hash::FxHashMap;
use crate::{Id, Node, Network, Receiver, Signal};

pub trait NetworkWithBackwardEdges: Network {
    fn node_outputs(&self, id: Id) -> impl Iterator<Item = Id> + '_;
    fn leafs(&self) -> impl Iterator<Item = Id> + '_;
}

pub struct ComputedNetworkWithBackwardEdges<'a, P: ?Sized> {
    network: &'a P,
    backward: FxHashMap<Id, Vec<Id>>,
    leafs: Vec<Id>,
}

impl<'a, P: Network + ?Sized> ComputedNetworkWithBackwardEdges<'a, P> {
    pub fn new(network: &'a P) -> Self {
        let mut backward = FxHashMap::default();
        let mut leafs = Vec::new();
        for (output_id, output) in network.iter() {
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
        Self { network, backward, leafs }
    }
}

impl<P: Network + ?Sized> Network for ComputedNetworkWithBackwardEdges<'_, P> {
    type Node = P::Node;

    fn outputs(&self) -> impl Iterator<Item=Signal> {
        self.network.outputs()
    }
    fn node(&self, id: Id) -> Self::Node {
        self.network.node(id)
    }
    fn iter(&self) -> impl Iterator<Item=(Id, Self::Node)> + '_ {
        self.network.iter()
    }
    fn send<R: Receiver<Node=Self::Node>>(&self, receiver: R) -> R::Result {
        self.network.send(receiver)
    }
}

impl<P: Network + ?Sized> NetworkWithBackwardEdges for ComputedNetworkWithBackwardEdges<'_, P> {
    fn node_outputs(&self, id: Id) -> impl Iterator<Item=Id> + '_ {
        self.backward.get(&id).into_iter().flat_map(|v| v.iter()).cloned()
    }
    fn leafs(&self) -> impl Iterator<Item=Id> + '_ {
        self.leafs.iter().cloned()
    }
}
