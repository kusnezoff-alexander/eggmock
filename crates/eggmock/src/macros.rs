#[macro_export]
macro_rules! define_network {
    ($(#[$meta:meta])* $vis:vis enum $name:ident {
        $($gate_str:literal = $gate:ident($fanin:literal, $mockturtle_create:ident, $mockturtle_is:ident)),+
    }) => {
        $crate::paste::paste! {
            $crate::egg::define_language! {
                $(#[$meta])*
                $vis enum [<$name Language>] {
                    Symbol(u64),
                    Const(bool),
                    "!" = Not($crate::egg::Id),
                    $($gate_str = $gate([$crate::egg::Id;$fanin])),+,
                }
            }

            $vis enum $name {
                Symbol(u64),
                Const(bool),
                Not(u64),
                $($gate([u64;$fanin])),+
            }

            impl $crate::Network for $name {
                type GateType = $crate::paste::paste!([<$name GateType>]);
                type Language = [<$name Language>];
                type TransferFFI = [<$name TransferFFI>];

                const TYPENAME: &'static str = stringify!([<$name:snake:lower>]);
                const GATE_TYPES: &'static [Self::GateType] = &[
                    $([<$name GateType>]::$gate),+
                ];

                fn map_ids(&self, map: impl Fn(u64) -> u64) -> Self {
                    match self {
                        Self::Symbol(name) => Self::Symbol(*name),
                        Self::Const(bool) => Self::Const(*bool),
                        Self::Not(id) => Self::Not(map(*id)),
                        $(Self::$gate(ids) => {
                            $crate::seq_macro::seq!(N in 0..$fanin {
                                Self::$gate([#(map(ids[N]),)*])
                            })
                        }),+
                    }
                }
            }

            #[derive(Copy, Clone, Eq, PartialEq, Hash)]
            $vis enum [<$name GateType>] {
                $($gate),+
            }

            impl $crate::GateType for [<$name GateType>] {
                type Network = $name;

                fn name(&self) -> &'static str {
                    match self {
                        $(Self::$gate => stringify!([<$gate:snake:lower>])),+
                    }
                }

                fn fanin(&self) -> u8 {
                    match self {
                        $(Self::$gate => $fanin),+
                    }
                }

                fn mockturtle_create(&self) -> &'static str {
                    match self {
                        $(Self::$gate => stringify!($mockturtle_create)),+
                    }
                }

                fn mockturtle_is(&self) -> &'static str {
                    match self {
                        $(Self::$gate => stringify!($mockturtle_is)),+
                    }
                }
            }

            impl From<$name> for [<$name Language>] {
                fn from(node: $name) -> Self {
                    match node {
                        $name::Symbol(name) => Self::Symbol(name),
                        $name::Const(value) => Self::Const(value),
                        $name::Not(id) => Self::Not($crate::egg::Id::from(id as usize)),
                        $($name::$gate(ids) => {
                            $crate::seq_macro::seq!(N in 0..$fanin {
                                Self::$gate([#($crate::egg::Id::from(ids[N] as usize),)*])
                            })
                        }),+
                    }
                }
            }

            impl From<[<$name Language>]> for $name {
                fn from(node: [<$name Language>]) -> Self {
                    match node {
                        [<$name Language>]::Symbol(name) => Self::Symbol(name),
                        [<$name Language>]::Const(value) => Self::Const(value),
                        [<$name Language>]::Not(id) => Self::Not(usize::from(id) as u64),
                        $([<$name Language>]::$gate(ids) => {
                            $crate::seq_macro::seq!(N in 0..$fanin {
                                Self::$gate([#(usize::from(ids[N]) as u64,)*])
                            })
                        }),+
                    }
                }
            }

            impl<A> $crate::NetworkTransfer<$name> for $crate::egg::EGraph<[<$name Language>], A>
            where
                A: $crate::egg::Analysis<[<$name Language>]>
            {
                fn create(&mut self, node: $name) -> u64 {
                    usize::from(self.add([<$name Language>]::from(node))) as u64
                }
            }

            #[repr(C)]
            $vis struct [<$name TransferFFI>] {
                pub create_symbol: extern "C" fn(*mut $crate::libc::c_void, name: u64) -> u64,
                pub create_constant: extern "C" fn (*mut $crate::libc::c_void, value: bool) -> u64,
                pub create_not: extern "C" fn (*mut $crate::libc::c_void, id: u64) -> u64,
                $(pub [<create_ $gate:snake:lower>]: $crate::seq_macro::seq!(N in 1..=$fanin {
                     extern "C" fn(*mut $crate::libc::c_void, #(id~N: u64,)*) -> u64
                })),+,
            }

            impl TransferFFI for [<$name TransferFFI>] {
                type Network = $name;

                fn new<T: $crate::AsNetworkTransfer<Self::Network> + Sized>() -> Self {
                    Self {
                        create_symbol: Self::create_symbol::<T>,
                        create_constant: Self::create_constant::<T>,
                        create_not: Self::create_not::<T>,
                        $([<create_ $gate:snake:lower>]: Self::[<create_ $gate:snake:lower>]::<T>),+,
                    }
                }

                fn create(&self, data: *mut libc::c_void, node: $name) -> u64 {
                    match node {
                        $name::Symbol(name) => (self.create_symbol)(data, name),
                        $name::Const(value) => (self.create_constant)(data, value),
                        $name::Not(id) => (self.create_not)(data, id),
                        $($name::$gate(ids) => {
                            $crate::seq_macro::seq!(N in 0..$fanin {
                                (self.[<create_ $gate:snake:lower>])(data, #(ids[N],)*)
                            })
                        }),+
                    }
                }
            }

            impl [<$name TransferFFI>] {
                extern "C" fn create_symbol<T: $crate::AsNetworkTransfer<$name> + Sized>(
                    data: *mut $crate::libc::c_void,
                    name: u64
                ) -> u64 {
                    unsafe { &mut *(data as *mut T) }.as_transfer().create($name::Symbol(name))
                }

                extern "C" fn create_constant<T: $crate::AsNetworkTransfer<$name> + Sized>(
                    data: *mut $crate::libc::c_void,
                    value: bool
                ) -> u64 {
                    unsafe { &mut *(data as *mut T) }.as_transfer().create($name::Const(value))
                }

                extern "C" fn create_not<T: $crate::AsNetworkTransfer<$name> + Sized>(
                    data: *mut $crate::libc::c_void,
                    id: u64
                ) -> u64 {
                    unsafe { &mut *(data as *mut T) }.as_transfer().create($name::Not(id))
                }

                $($crate::seq_macro::seq!(N in 1..=$fanin {
                    pub extern "C" fn [<create_ $gate:snake:lower>]<T: $crate::AsNetworkTransfer<$name> + Sized>(
                        data: *mut $crate::libc::c_void
                        #(, id~N: u64)*
                    ) -> u64 {
                        unsafe { &mut *(data as *mut T) }
                            .as_transfer()
                            .create($name::$gate([#(id~N,)*]))
                    }
                });)+
            }
        }
    };
}
