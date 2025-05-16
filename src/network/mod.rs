use crate::Receiver;

use super::ReceiverFFI;
use egg::Language;
use rustc_hash::{FxHashMap, FxHashSet};
use std::hash::Hash;

mod backwards;

pub use backwards::*;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[repr(C)]
/// References a node in a network.
pub struct Id(u32);

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[repr(C)]
/// References a node by its id with a flag that indicates whether the signal from this node is
/// inverted.
pub struct Signal(u32);

/// Describes a node of a logic network. This includes PIs, constant nodes and gates.
pub trait Node: 'static + Sized + Clone + Hash + Eq {
    /// The type that contains descriptions of the gate types in this network.
    type Gates: GateType<Node = Self>;
    /// An *egg* Language that can represent networks of this type.
    type Language: NetworkLanguage<Node = Self>;
    /// The type that contains function pointers to transfer a network of this type to or from a C++
    /// library using the [`Receiver`](crate::Receiver) trait on the Rust-side.
    type ReceiverFFI<R>: ReceiverFFI<Node = Self, Result = R>;

    /// A snake_case name for this network type, which is used to name things in the generated C
    /// code (e.g. `"aig"` for AIGs).
    const NTK_TYPENAME: &'static str;
    /// The name of the equivalent network type in *mockturtle* (e.g. `aig_network` for
    /// [`Aig`](crate::Aig)).
    const NTK_MOCKTURTLE_TYPENAME: &'static str;
    /// The header file for this network type in *mockturtle* (e.g. `mockturtle/networks/aig.hpp`)
    const NTK_MOCKTURTLE_INCLUDE: &'static str;

    /// Returns the same type of node but with the input signals mapped with the given function.
    fn map_input_signals(&self, map: impl FnMut(Signal) -> Signal) -> Self;
    /// Returns the input signals of this node. May be empty for non-gate nodes (such as constants
    /// or PIs).
    fn inputs(&self) -> &[Signal];
    /// Returns the same type of node but with the ids of each input signal replaced by the signal
    /// given by the mapping function. See also [`Signal::map_id`].
    fn map_input_ids(&self, mut map: impl FnMut(Id) -> Signal) -> Self {
        self.map_input_signals(|signal| signal.map_id(&mut map))
    }
    fn is_leaf(&self) -> bool {
        self.inputs().is_empty()
    }
}

/// Contains the [`Language`] type that can represent a Network.
pub trait NetworkLanguage: Language {
    type Node: Node;

    /// Creates an instance of this type given a node of the network. The input signals are mapped
    /// to child ids with the given mapper.
    fn from_node(
        node: Self::Node,
        signal_mapper: impl FnMut(Signal) -> egg::Id,
    ) -> Self;
    /// Creates a network node from this EGraph node. The child ids are mapped to signals with the
    /// given mapper. This mapper will usually resolve the nots before the next real network node.
    ///
    /// Returns [`None`] if this EGraph node is a not.
    fn to_node(
        &self,
        id_mapper: impl FnMut(egg::Id) -> Signal,
    ) -> Option<Self::Node>;

    /// Returns true iff this node is a not.
    fn is_not(&self) -> bool;
    /// Creates a new not node with the given child id.
    fn not(id: egg::Id) -> Self;
}

impl Signal {
    const NOT_MASK: u32 = 1 << 31;

    pub fn new(id: Id, inverted: bool) -> Signal {
        Signal(id.0).maybe_invert(inverted)
    }

    pub fn is_inverted(&self) -> bool {
        self.0 & Self::NOT_MASK != 0
    }
    pub fn maybe_invert(&self, invert: bool) -> Signal {
        Signal(self.0 ^ ((invert as u32) << 31))
    }
    pub fn invert(&self) -> Signal {
        self.maybe_invert(true)
    }
    pub fn node_id(&self) -> Id {
        Id(self.0 & !Self::NOT_MASK)
    }
    /// Replaces the id of this signal with the given signal. That is, the id of the returned signal
    /// is the same as the id of the parameter signal and the returned signal will be inverted if
    /// exactly one of the two given signals is inverted.
    pub fn replace_id(&self, signal: Signal) -> Signal {
        Signal::new(signal.node_id(), self.is_inverted() ^ signal.is_inverted())
    }
    /// Performs [`Self::replace_id`] with the signal given by the mapping function for this
    /// signal's id.
    pub fn map_id(&self, map: impl FnOnce(Id) -> Signal) -> Signal {
        self.replace_id(map(self.node_id()))
    }
}

/// Contains description of the gates in a network, which is used for code generation.
pub trait GateType: 'static + Sized {
    type Node: Node<Gates = Self>;

    /// Contains all gate types of the associated Network type.
    const VARIANTS: &'static [Self];

    /// Returns the snake_case name of this gate type, which is used in code generation (e.g.
    /// `"and"` for an AND gate)
    fn name(&self) -> &'static str;
    /// Returns the number of inputs that a gate of this type has (2 for AND, 3 for MAJ etc.).
    fn fanin(&self) -> u8;

    /// Returns the name of the method on the *mockturtle* network implementation that creates a
    /// gate of this type from [`fanin`](Self::fanin) signals (e.g. `"create_and"`).
    fn mockturtle_create(&self) -> &'static str;
    /// Returns the name of the method on the *mockturtle* network implementation that checks
    /// whether a given node ID belongs to a gate of this type (e.g. `"is_and"`).
    fn mockturtle_is(&self) -> &'static str;
}

impl From<egg::Id> for Id {
    fn from(value: egg::Id) -> Self {
        Id(usize::from(value) as u32)
    }
}

impl From<Id> for egg::Id {
    fn from(value: Id) -> Self {
        egg::Id::from(value.0 as usize)
    }
}

impl From<u32> for Id {
    fn from(id: u32) -> Self {
        Id(id)
    }
}

impl From<Id> for u32 {
    fn from(id: Id) -> Self {
        id.0
    }
}

/// A type that contains a logic network.
pub trait Network {
    type Node: Node;

    /// Returns an iterator containing the ids of the output nodes of the underlying network.
    fn outputs(&self) -> impl Iterator<Item = Signal>;
    /// Returns the node with the given id.
    fn node(&self, id: Id) -> Self::Node;
    /// Returns an iterator over all nodes that are reachable from an output and their ids.
    fn iter(&self) -> impl Iterator<Item = (Id, Self::Node)> + '_ {
        NetworkNodeIterator {
            network: self,
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

    fn with_backward_edges(&self) -> impl NetworkWithBackwardEdges<Node = Self::Node> + '_ {
        ComputedNetworkWithBackwardEdges::new(self)
    }
}

struct NetworkNodeIterator<'a, P: ?Sized> {
    network: &'a P,
    visited: FxHashSet<Id>,
    remaining: Vec<Id>,
}

impl<P: Network + ?Sized> Iterator for NetworkNodeIterator<'_, P> {
    type Item = (Id, P::Node);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let node_id = self.remaining.pop()?;
            if !self.visited.insert(node_id) {
                continue;
            }
            let node = self.network.node(node_id);
            self.remaining.extend(node.inputs().iter().map(|s| s.node_id()));
            break Some((node_id, node));
        }
    }
}
