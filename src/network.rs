use super::ReceiverFFI;
use egg::Language;

/// Contains the equivalent [`Language`] type associated with a [`Network`] type
pub trait NetworkLanguage: Language + From<Self::Network> {
    type Network: Network<Language = Self> + From<Self>;
}

/// Describes a node of a logic network.
///
/// This trait is usually implemented by creating a `Network` type using the
/// [`define_network`](crate::define_network) macro.
pub trait Network: 'static + Sized {
    /// The type that contains descriptions of the special gates in this network.
    type GateType: GateType<Network = Self>;
    /// A Language that is equivalent to this type (i.e. can be converted from and to an instance of
    /// this type)
    type Language: NetworkLanguage<Network = Self>;
    /// The type that contains function pointers to transfer a network of this type to or from a C
    /// library using the [`Receiver`](crate::Receiver) trait on the Rust-side.
    type ReceiverFFI<R>: ReceiverFFI<Self, Result = R>;

    /// A snake_case name for this network type, which is used to name things in the generated C
    /// code (e.g. `"aig"` for AIGs).
    const TYPENAME: &'static str;
    /// The name of the equivalent network type in mockturtle (e.g. `"aig_network"` for
    /// [`Aig`](crate::Aig)).
    const MOCKTURTLE_TYPENAME: &'static str;

    /// Returns the same type of node but with the IDs of the inputs mapped with the given function.
    fn map_inputs(&self, map: impl Fn(u64) -> u64) -> Self;
    /// Returns the input IDs of this node. May be empty for non-gate nodes (such as constants or
    /// "primary" inputs).
    fn inputs(&self) -> &[u64];
}

/// Describes the special node types of a logic network.
///
/// This includes only the "special" nodes such as ANDs in AIGs or MAJ in MIGs. NOT-Gates are
/// considered normal (which has to do with the fact that they are handled differently than the
/// aforementioned in *mockturtle* and therefore the FFI code generation has to handle them
/// differently as well).
pub trait GateType: 'static + Sized {
    type Network: Network<GateType = Self>;
    /// Contains all special gate types of the associated Network type.
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
