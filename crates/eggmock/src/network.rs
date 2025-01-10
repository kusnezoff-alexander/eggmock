use egg::Language;

pub trait Network: From<Self::Language> {
    type GateType: GateType<Network = Self>;
    type Language: Language + From<Self>;
    type TransferFFI: TransferFFI<Network = Self>;

    const TYPENAME: &'static str;
    const GATE_TYPES: &'static [Self::GateType];
    const MOCKTURTLE_TYPENAME: &'static str;

    fn map_ids(&self, map: impl Fn(u64) -> u64) -> Self;
}

pub trait GateType: Sized + 'static {
    type Network: Network<GateType = Self>;

    fn name(&self) -> &'static str;
    fn fanin(&self) -> u8;

    fn mockturtle_create(&self) -> &'static str;
    fn mockturtle_is(&self) -> &'static str;
}

pub trait NetworkTransfer<N: Network> {
    fn create(&mut self, node: N) -> u64;
}

pub trait TransferFFI {
    type Network: Network;

    fn new<T: AsNetworkTransfer<Self::Network>>() -> Self;
    fn create(&self, data: *mut libc::c_void, node: Self::Network) -> u64;
}

pub trait AsNetworkTransfer<N: Network> {
    fn as_transfer(&mut self) -> &mut impl NetworkTransfer<N>;
}
