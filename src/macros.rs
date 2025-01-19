#[macro_export]
macro_rules! define_network {
    ($(#[$meta:meta])* $vis:vis enum $mockturtle_ntk:literal = $name:ident {
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

            #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
            $vis enum $name {
                Symbol(u64),
                Const(bool),
                Not(u64),
                $($gate([u64;$fanin])),+
            }

            impl $crate::Network for $name {
                type GateType = $crate::paste::paste!([<$name GateType>]);
                type Language = [<$name Language>];
                type ReceiverFFI<R> = [<$name ReceiverFFI>]<R>;

                const TYPENAME: &'static str = stringify!([<$name:snake:lower>]);
                const GATE_TYPES: &'static [Self::GateType] = &[
                    $([<$name GateType>]::$gate),+
                ];
                const MOCKTURTLE_TYPENAME: &'static str = concat!($mockturtle_ntk, "_network");

                fn map_children(&self, map: impl Fn(u64) -> u64) -> Self {
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

                fn children(&self) -> &[u64] {
                    match self {
                        Self::Symbol(_) => &[],
                        Self::Const(_) => &[],
                        Self::Not(id) => std::slice::from_ref(id),
                        $(Self::$gate(ids) => ids),+
                    }
                }
            }

            impl NetworkLanguage for [<$name Language>] {
                type Network = $name;
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

            #[repr(C)]
            $vis struct [<$name ReceiverFFI>]<R> {
                data: *mut $crate::libc::c_void,
                create_symbol: extern "C" fn(*mut $crate::libc::c_void, name: u64) -> u64,
                create_constant: extern "C" fn (*mut $crate::libc::c_void, value: bool) -> u64,
                create_not: extern "C" fn (*mut $crate::libc::c_void, id: u64) -> u64,
                $([<create_ $gate:snake:lower>]: $crate::seq_macro::seq!(N in 1..=$fanin {
                     extern "C" fn(*mut $crate::libc::c_void, #(id~N: u64,)*) -> u64
                })),+,
                done: extern "C" fn (*mut $crate::libc::c_void, outputs: *const u64, outputs_size: usize) -> R,
            }

            impl<R> $crate::ReceiverFFI for [<$name ReceiverFFI>]<R> {
                fn new<Recv>(receiver: Recv) -> Self
                where
                    Recv: $crate::Receiver<Network = $name, Result = R> + 'static
                {
                    let data = Box::into_raw(Box::new(receiver));
                    Self {
                        data: data as *mut $crate::libc::c_void,
                        create_symbol: Self::create_symbol::<Recv>,
                        create_constant: Self::create_constant::<Recv>,
                        create_not: Self::create_not::<Recv>,
                        $([<create_ $gate:snake:lower>]: Self::[<create_ $gate:snake:lower>]::<Recv>),+,
                        done: Self::done::<Recv>,
                    }
                }
            }

            impl<R> $crate::Receiver for [<$name ReceiverFFI>]<R> {
                type Network = $name;
                type Result = R;

                fn create_node(&mut self, node: $name) -> u64 {
                    match node {
                        $name::Symbol(name) => (self.create_symbol)(self.data, name),
                        $name::Const(value) => (self.create_constant)(self.data, value),
                        $name::Not(id) => (self.create_not)(self.data, id),
                        $($name::$gate(ids) => {
                            $crate::seq_macro::seq!(N in 0..$fanin {
                                (self.[<create_ $gate:snake:lower>])(self.data, #(ids[N],)*)
                            })
                        }),+
                    }
                }

                fn done(self, outputs: &[u64]) -> R {
                    (self.done)(self.data, outputs.as_ptr(), outputs.len())
                }
            }

            impl<R> [<$name ReceiverFFI>]<R> {
                extern "C" fn create_symbol<Recv>(
                    data: *mut $crate::libc::c_void,
                    name: u64
                ) -> u64
                where
                    Recv: $crate::Receiver<Network = $name, Result = R> + 'static
                {
                    unsafe { &mut *(data as *mut Recv) }.create_node($name::Symbol(name))
                }

                extern "C" fn create_constant<Recv>(
                    data: *mut $crate::libc::c_void,
                    value: bool
                ) -> u64
                where
                    Recv: $crate::Receiver<Network = $name, Result = R> + 'static
                {
                    unsafe { &mut *(data as *mut Recv) }.create_node($name::Const(value))
                }

                extern "C" fn create_not<Recv>(
                    data: *mut $crate::libc::c_void,
                    id: u64
                ) -> u64
                where
                    Recv: $crate::Receiver<Network = $name, Result = R> + 'static
                {
                    unsafe { &mut *(data as *mut Recv) }.create_node($name::Not(id))
                }

                $($crate::seq_macro::seq!(N in 1..=$fanin {
                    pub extern "C" fn [<create_ $gate:snake:lower>]<Recv>(
                        data: *mut $crate::libc::c_void
                        #(, id~N: u64)*
                    ) -> u64
                    where
                        Recv: $crate::Receiver<Network = $name, Result = R> + 'static
                    {
                        unsafe { &mut *(data as *mut Recv) }.create_node($name::$gate([#(id~N,)*]))
                    }
                });)+

                extern "C" fn done<Recv: $crate::Receiver<Network = $name, Result = R>>(
                    data: *mut $crate::libc::c_void,
                    outputs: *const u64,
                    outputs_size: usize,
                ) -> R
                where
                    Recv: $crate::Receiver<Network = $name, Result = R> + 'static
                {
                    let outputs = if outputs_size == 0 {
                        &[]
                    } else {
                        unsafe { std::slice::from_raw_parts(outputs, outputs_size) }
                    };
                    unsafe { Box::from_raw(data as *mut Recv) }.done(outputs)
                }
            }
        }
    };
}
