use super::{Id, Node, Signal};
use rustc_hash::{FxHashMap, FxHashSet};

/// A type that can receive nodes of a logic network and produce some result from it.
pub trait Receiver: Sized {
    type Node: Node;
    type Result;

    /// Creates the given signal. Returns the id of the newly created signal.
    fn create_node(&mut self, node: Self::Node) -> Signal;
    /// Creates the result from the previously transferred nodes where `outputs` contains the output
    /// signals.
    fn done(self, outputs: &[Signal]) -> Self::Result;
    /// Maps the result of this Receiver using the given function.
    fn map<Res2, F>(self, map: F) -> impl Receiver<Node = Self::Node, Result = Res2>
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
pub trait Provider {
    type Node: Node;

    /// Returns an iterator containing the ids of the output nodes of the underlying network.
    fn outputs(&self) -> impl Iterator<Item = Signal>;
    /// Returns the node with the given id.
    fn node(&self, id: Id) -> Self::Node;
    /// Returns an iterator over all nodes that are reachable from an output and their ids.
    fn iter(&self) -> impl Iterator<Item = (Id, Self::Node)> + '_ {
        ProviderNodeIterator {
            provider: self,
            visited: FxHashSet::default(),
            remaining: Vec::from_iter(self.outputs().map(|s| s.node_id())),
        }
    }
    /// Sends this network to the given receiver.
    fn send<R: Receiver<Node = Self::Node>>(&self, mut receiver: R) -> R::Result {
        let mut src_to_dest: FxHashMap<Id, Signal> = FxHashMap::default();
        let mut path = Vec::new();
        for signal in self.outputs() {
            let mut node_id = signal.node_id();
            let mut node = self.node(node_id);
            let mut known_inputs = 0;
            loop {
                if known_inputs == node.inputs().len() || src_to_dest.contains_key(&node_id) {
                    if known_inputs == node.inputs().len() {
                        let dest_node = node.map_input_ids(|id| src_to_dest[&id]);
                        let dest_signal = receiver.create_node(dest_node);
                        src_to_dest.insert(node_id, dest_signal);
                    }
                    if path.is_empty() {
                        break;
                    }
                    (node_id, node, known_inputs) = path.pop().unwrap();
                    known_inputs += 1;
                } else {
                    let child_id = node.inputs()[known_inputs].node_id();
                    path.push((node_id, node, known_inputs));
                    node_id = child_id;
                    node = self.node(node_id);
                    known_inputs = 0;
                }
            }
        }
        let outputs = Vec::from_iter(
            self.outputs()
                .map(|signal| signal.map_id(|id| src_to_dest[&id])),
        );
        receiver.done(outputs.as_slice())
    }
}

struct ProviderNodeIterator<'a, P: ?Sized> {
    provider: &'a P,
    visited: FxHashSet<Id>,
    remaining: Vec<Id>,
}

impl<P: Provider + ?Sized> Iterator for ProviderNodeIterator<'_, P> {
    type Item = (Id, P::Node);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let node_id = self.remaining.pop()?;
            if !self.visited.insert(node_id) {
                continue;
            }
            let node = self.provider.node(node_id);
            self.remaining.extend(node.inputs().iter().map(|s| s.node_id()));
            break Some((node_id, node));
        }
    }
}

pub trait ReceiverFFI: Receiver {
    fn new<R>(receiver: R) -> Self
    where
        R: Receiver<Node = Self::Node, Result = Self::Result> + 'static;
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
    type Node = O::Node;
    type Result = R;
    fn create_node(&mut self, node: Self::Node) -> Signal {
        self.original.create_node(node)
    }
    fn done(self, outputs: &[Signal]) -> Self::Result {
        (self.map)(self.original.done(outputs))
    }
}
