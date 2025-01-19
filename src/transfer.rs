use super::{Network, NetworkLanguage};
use egg::{Analysis, CostFunction, EGraph, Extractor, Id};
use rustc_hash::FxHashMap;

pub trait Receiver: Sized {
    type Network: Network;
    type Result;

    fn create_node(&mut self, node: Self::Network) -> u64;
    fn done(self, outputs: &[u64]) -> Self::Result;
    fn map<Res2, F>(self, map: F) -> impl Receiver<Network = Self::Network, Result = Res2>
    where
        F: FnOnce(Self::Result) -> Res2,
    {
        MappedReceiver {
            original: self,
            map,
        }
    }
}

pub trait Provider {
    type Network: Network;

    fn outputs(&self) -> impl Iterator<Item = u64>;
    fn node(&self, id: u64) -> Self::Network;
    fn send<R: Receiver<Network = Self::Network>>(&self, mut receiver: R) -> R::Result {
        let mut src_to_dest_id = FxHashMap::default();
        let mut path = Vec::new();
        for node_id in self.outputs() {
            let mut node_id = node_id;
            let mut node = self.node(node_id);
            let mut known_children = 0;
            loop {
                if known_children == node.children().len() || src_to_dest_id.contains_key(&node_id)
                {
                    if known_children == node.children().len() {
                        let dest_node = node.map_children(|child| src_to_dest_id[&child]);
                        let dest_node_id = receiver.create_node(dest_node);
                        src_to_dest_id.insert(node_id, dest_node_id);
                    }
                    if path.is_empty() {
                        break;
                    }
                    (node_id, node, known_children) = path.pop().unwrap();
                    known_children += 1;
                } else {
                    let child_id = node.children()[known_children];
                    path.push((node_id, node, known_children));
                    node_id = child_id;
                    node = self.node(node_id);
                    known_children = 0;
                }
            }
        }
        let outputs = Vec::from_iter(self.outputs().map(|src_id| src_to_dest_id[&src_id]));
        receiver.done(outputs.as_slice())
    }
}

pub trait ReceiverFFI: Receiver {
    fn new<R>(receiver: R) -> Self
    where
        R: Receiver<Network = Self::Network, Result = Self::Result> + 'static;
}

impl<L: NetworkLanguage, A: Analysis<L>> Receiver for EGraph<L, A> {
    type Network = L::Network;
    type Result = (Self, Vec<Id>);

    fn create_node(&mut self, node: Self::Network) -> u64 {
        usize::from(self.add(L::from(node))) as u64
    }

    fn done(self, outputs: &[u64]) -> Self::Result {
        let mut outputs_vec = Vec::with_capacity(outputs.len());
        outputs_vec.extend(
            outputs
                .iter()
                .map(|output_id| Id::from(*output_id as usize)),
        );
        (self, outputs_vec)
    }
}

struct MappedReceiver<Original, Function> {
    original: Original,
    map: Function,
}

impl<O, R, F> Receiver for MappedReceiver<O, F>
where
    O: Receiver,
    F: FnOnce(O::Result) -> R,
{
    type Network = O::Network;
    type Result = R;
    fn create_node(&mut self, node: Self::Network) -> u64 {
        self.original.create_node(node)
    }
    fn done(self, outputs: &[u64]) -> Self::Result {
        (self.map)(self.original.done(outputs))
    }
}

impl<'a, L: NetworkLanguage, CF: CostFunction<L>, A: Analysis<L>> Provider
    for (Extractor<'a, CF, L, A>, Vec<Id>)
{
    type Network = L::Network;

    fn outputs(&self) -> impl Iterator<Item = u64> {
        self.1.iter().map(|o| usize::from(*o) as u64)
    }

    fn node(&self, id: u64) -> Self::Network {
        Self::Network::from(self.0.find_best_node(Id::from(id as usize)).clone())
    }
}
