use super::{Network, NetworkLanguage};
use egg::{Analysis, CostFunction, EGraph, Extractor, Id};
use rustc_hash::{FxHashMap, FxHashSet};
use std::marker::PhantomData;

/// A type that can receive a logic network and produce some result from it.
///
/// This trait is implemented for [`EGraph`s](EGraph) with [`NetworkLanguage`s](NetworkLanguage), so
/// that EGraphs can be easily received from C++ code.
pub trait Receiver<N: Network>: Sized {
    type Result;

    /// Creates the given node. Returns the id of the newly created node.
    fn create_node(&mut self, node: N) -> u64;
    /// Creates the result from the previously transferred nodes where `outputs` contains the ids of
    /// the nodes that generate the output signals of the network.
    fn done(self, outputs: &[u64]) -> Self::Result;
    /// Maps the result of this Receiver using the given function.
    fn map<Res2, F>(self, map: F) -> impl Receiver<N, Result = Res2>
    where
        F: FnOnce(Self::Result) -> Res2,
    {
        MappedReceiver {
            original: self,
            map,
        }
    }
}

/// A type that contains a logic network.
///
/// This trait is readily implemented for tuples of [`Extractor`s](Extractor) on
/// [`NetworkLanguage`s](NetworkLanguage) and `Vec<Id>` where the nodes are taken as the best enodes
/// of the respective eclass and the output ids are taken from the `Vec`. IDs are converted via
/// `egg::Id::from(id as usize)` and vice versa.
pub trait Provider<N: Network> {
    /// Returns an iterator containing the ids of the output nodes of the underlying network.
    fn outputs(&self) -> impl Iterator<Item = u64>;
    /// Returns the node with the given id.
    fn node(&self, id: u64) -> N;
    /// Returns an iterator over all nodes that are reachable from an output and their ids.
    fn iter(&self) -> impl Iterator<Item = (u64, N)> + '_ {
        ProviderNodeIterator {
            provider: self,
            visited: FxHashSet::default(),
            remaining: Vec::from_iter(self.outputs()),
            _n: PhantomData,
        }
    }
    /// Sends this network to the given receiver.
    fn send<R: Receiver<N>>(&self, mut receiver: R) -> R::Result {
        let mut src_to_dest_id = FxHashMap::default();
        let mut path = Vec::new();
        for node_id in self.outputs() {
            let mut node_id = node_id;
            let mut node = self.node(node_id);
            let mut known_inputs = 0;
            loop {
                if known_inputs == node.inputs().len() || src_to_dest_id.contains_key(&node_id) {
                    if known_inputs == node.inputs().len() {
                        let dest_node = node.map_inputs(|child| src_to_dest_id[&child]);
                        let dest_node_id = receiver.create_node(dest_node);
                        src_to_dest_id.insert(node_id, dest_node_id);
                    }
                    if path.is_empty() {
                        break;
                    }
                    (node_id, node, known_inputs) = path.pop().unwrap();
                    known_inputs += 1;
                } else {
                    let child_id = node.inputs()[known_inputs];
                    path.push((node_id, node, known_inputs));
                    node_id = child_id;
                    node = self.node(node_id);
                    known_inputs = 0;
                }
            }
        }
        let outputs = Vec::from_iter(self.outputs().map(|src_id| src_to_dest_id[&src_id]));
        receiver.done(outputs.as_slice())
    }
}

struct ProviderNodeIterator<'a, N, P: ?Sized> {
    provider: &'a P,
    visited: FxHashSet<u64>,
    remaining: Vec<u64>,
    _n: PhantomData<fn() -> N>,
}

impl<'a, N: Network, P: Provider<N> + ?Sized> Iterator for ProviderNodeIterator<'a, N, P> {
    type Item = (u64, N);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let node_id = self.remaining.pop()?;
            if !self.visited.insert(node_id) {
                continue;
            }
            let node = self.provider.node(node_id);
            self.remaining.extend_from_slice(node.inputs());
            break Some((node_id, node));
        }
    }
}

pub trait ReceiverFFI<N: Network>: Receiver<N> {
    fn new<R>(receiver: R) -> Self
    where
        R: Receiver<N, Result = Self::Result> + 'static;
}

impl<L: NetworkLanguage, A: Analysis<L>> Receiver<L::Network> for EGraph<L, A> {
    type Result = (Self, Vec<Id>);

    fn create_node(&mut self, node: L::Network) -> u64 {
        usize::from(self.add(L::from(node))) as u64
    }

    fn done(self, outputs: &[u64]) -> Self::Result {
        (
            self,
            Vec::from_iter(outputs.iter().map(|id| Id::from(*id as usize))),
        )
    }
}

struct MappedReceiver<Original, Function> {
    original: Original,
    map: Function,
}

impl<N, O, R, F> Receiver<N> for MappedReceiver<O, F>
where
    N: Network,
    O: Receiver<N>,
    F: FnOnce(O::Result) -> R,
{
    type Result = R;
    fn create_node(&mut self, node: N) -> u64 {
        self.original.create_node(node)
    }
    fn done(self, outputs: &[u64]) -> Self::Result {
        (self.map)(self.original.done(outputs))
    }
}

impl<'a, L: NetworkLanguage, CF: CostFunction<L>, A: Analysis<L>> Provider<L::Network>
    for (Extractor<'a, CF, L, A>, Vec<Id>)
{
    fn outputs(&self) -> impl Iterator<Item = u64> {
        self.1.iter().map(|o| usize::from(*o) as u64)
    }

    fn node(&self, id: u64) -> L::Network {
        L::Network::from(self.0.find_best_node(Id::from(id as usize)).clone())
    }
}
