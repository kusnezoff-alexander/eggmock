use super::ReceiverFFI;
use egg::Language;
use std::hash::Hash;

mod provider;
mod backwards;

pub use provider::*;
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

pub trait Network: Sized + 'static {
    /// The node type of this network.
    type Node: Node<Network = Self>;
    /// The type that contains descriptions of the gate types in this network.
    type Gates: GateType<Network = Self>;
    /// An *egg* Language that can represent networks of this type.
    type Language: NetworkLanguage<Network = Self>;
    /// The type that contains function pointers to transfer a network of this type to or from a C++
    /// library using the [`Receiver`](crate::Receiver) trait on the Rust-side.
    type ReceiverFFI<R>: ReceiverFFI<Node = Self::Node, Result = R>;

    /// A snake_case name for this network type, which is used to name things in the generated C
    /// code (e.g. `"aig"` for AIGs).
    const TYPENAME: &'static str;
    /// The name of the equivalent network type in *mockturtle* (e.g. `aig_network` for
    /// [`Aig`](crate::Aig)).
    const MOCKTURTLE_TYPENAME: &'static str;
    /// The header file for this network type in *mockturtle* (e.g. `mockturtle/networks/aig.hpp`)
    const MOCKTURTLE_INCLUDE: &'static str;
}

/// Describes a node of a logic network. This includes PIs, constant nodes and gates.
pub trait Node: 'static + Sized + Clone + Hash + Eq {
    type Network: Network<Node = Self>;

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
    type Network: Network;

    /// Creates an instance of this type given a node of the network. The input signals are mapped
    /// to child ids with the given mapper.
    fn from_node(
        node: <Self::Network as Network>::Node,
        signal_mapper: impl FnMut(Signal) -> egg::Id,
    ) -> Self;
    /// Creates a network node from this EGraph node. The child ids are mapped to signals with the
    /// given mapper. This mapper will usually resolve the nots before the next real network node.
    ///
    /// Returns [`None`] if this EGraph node is a not.
    fn to_node(
        &self,
        id_mapper: impl FnMut(egg::Id) -> Signal,
    ) -> Option<<Self::Network as Network>::Node>;

    /// Returns true iff this node is a not.
    fn is_not(&self) -> bool;
    /// Creates a new not node with the given child id.
    fn not(id: egg::Id) -> Self;
}

impl Signal {
    const NOT_MASK: u32 = 1 << 31;

    pub fn new(id: Id, inverted: bool) -> Signal {
        Signal(id.0 ^ ((inverted as u32) << 31))
    }

    pub fn is_inverted(&self) -> bool {
        self.0 & Self::NOT_MASK != 0
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
    type Network: Network<Gates = Self>;

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
