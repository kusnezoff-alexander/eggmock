use egg::Language;
use super::ReceiverFFI;

pub trait NetworkLanguage: Language + From<Self::Network> {
    type Network: Network<Language = Self> + From<Self>;
}

pub trait Network: 'static + Sized {
    type GateType: GateType<Network = Self>;
    type Language: NetworkLanguage<Network = Self>;
    type ReceiverFFI<R>: ReceiverFFI<Network = Self, Result = R>;

    const TYPENAME: &'static str;
    const GATE_TYPES: &'static [Self::GateType];
    const MOCKTURTLE_TYPENAME: &'static str;

    fn map_children(&self, map: impl Fn(u64) -> u64) -> Self;
    fn children(&self) -> &[u64];
}

pub trait GateType: 'static + Sized {
    type Network: Network<GateType = Self>;

    fn name(&self) -> &'static str;
    fn fanin(&self) -> u8;

    fn mockturtle_create(&self) -> &'static str;
    fn mockturtle_is(&self) -> &'static str;
}