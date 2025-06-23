/// # Example
/// ```no_run
/// eggmock::define_network! {
///     pub enum "xag" = Xag {
///         // 2: fanin
///         "*" = And(2),
///         "xor" = Xor(2)
///         "xor4" =
///     }
/// }
/// ```
///
/// Auto-implements:
/// - [<$name Language>] (using [`egg::define_language`])
/// - `enum $name`: holds language-specific node-types
///     - eg for `Aig`: Input-Gates (=id of nodes): input, false, and gates: Not, And
/// - implements [`Node`] for `$name`
/// - implements  [`NetworkLanguage`] for `[<$name Language>]`, eg for `AigLanguage`
///     - implemented operands: `"f"` (False), `Input(64)`
///     - implemented operators: `"!"` (NOT), language-specific gates (eg `"maj"` for MIG, `"and"` for AIG)
/// - implements [`GateType`] for `[<$name GateType]`
/// - implements [`ReceiverFFI`] for `[<$name ReceiverFFI>]<R>`
/// - implements [`Receiver` for [<$name ReceiverFFI>]<R>`
///
/// NOTE: see [paste](https://docs.rs/paste/latest/paste/) for understanding `[<...>]` notation
#[macro_export]
macro_rules! define_network {(
        $(#[$meta:meta])* $vis:vis enum $mockturtle_ntk:literal = $name:ident {
            // Binary gates
            gates {
                $($gate_str:literal = $gate:ident($fanin:literal)),* $(,)?
            }
            // N-ary gates
            $(nary_gates {
                $($gate_nary_str:literal = $gate_nary:ident($fanin_nary:literal)),* $(,)?
            })*
        }
    ) => {
        $crate::paste::paste! {
            $crate::egg::define_language! {
                /// Define Language (to be used in egg)
                /// - basically a string-representation of the graph
                $(#[$meta])*
                $vis enum [<$name Language>] {
                    Input(u64), // TODO: change `u64` to `Signal`??
                    "f" = False,
                    "!" = Not($crate::egg::Id),
                    $($gate_str = $gate([$crate::egg::Id;$fanin])),+,
                    $($gate_nary_str = $gate_nary([$crate::egg::Id;$fanin_nary])),*
                }
            }

            /// The network `$name` consists of: inputs, false and all network-specific gates (eg AND for AIG)
            #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
            $vis enum $name {
                Input(u64),
                False,
                $($gate([$crate::Signal;$fanin])),+
                $($gate_nary([$crate::Signal;$fanin_nary])),*
            }

            impl $crate::Node for $name {
                type Gates = $crate::paste::paste!([<$name GateType>]);
                type Language = [<$name Language>];
                type ReceiverFFI<R> = [<$name ReceiverFFI>]<R>;

                const NTK_TYPENAME: &'static str = stringify!([<$name:snake:lower>]);
                const NTK_MOCKTURTLE_TYPENAME: &'static str = concat!($mockturtle_ntk, "_network");
                const NTK_MOCKTURTLE_INCLUDE: &'static str = concat!(
                    "mockturtle/networks/", $mockturtle_ntk, ".hpp"
                );

                fn map_input_signals(&self, mut map: impl FnMut(Signal) -> Signal) -> Self {
                    match self {
                        Self::Input(name) => Self::Input(*name),
                        Self::False => Self::False,
                        $(Self::$gate(signals) => {
                            $crate::seq_macro::seq!(N in 0..$fanin {
                                Self::$gate([#(map(signals[N]),)*])
                            })
                        }),+
                    }
                }

                fn inputs(&self) -> &[Signal] {
                    match self {
                        Self::Input(_) => &[],
                        Self::False => &[],
                        $(Self::$gate(ids) => ids),+
                    }
                }
            }

            /// For Conversion btw representation in `mockturtle` (nodes) and representation in `egg` (as Signals)
            impl NetworkLanguage for [<$name Language>] {
                type Node = $name;

                fn from_node(
                    node: $name,
                    mut signal_mapper: impl FnMut(Signal) -> egg::Id,
                ) -> Self {
                    match node {
                        $name::Input(id) => Self::Input(id),
                        $name::False => Self::False,
                        $(
                        $name::$gate(ids) => Self::$gate(
                            $crate::seq_macro::seq!(N in 0..$fanin {
                                [#(signal_mapper(ids[N]),)*]
                            })
                        )
                        ),+
                    }
                }

                fn to_node(
                    &self,
                    mut id_mapper: impl FnMut(egg::Id) -> Signal
                ) -> Option<$name> {
                    match self {
                        Self::Input(id) => Some($name::Input(*id)),
                        Self::False => Some($name::False),
                        Self::Not(_) => None,
                        $(
                        Self::$gate(ids) => Some($name::$gate(
                            $crate::seq_macro::seq!(N in 0..$fanin {
                                [#(id_mapper(ids[N]),)*]
                            })
                        ))
                        ),+
                    }
                }

                fn is_not(&self) -> bool {
                    match self {
                        Self::Not(_) => true,
                        _ => false,
                    }
                }
                fn not(id: $crate::egg::Id) -> Self {
                    Self::Not(id)
                }
            }

            /// Network-specific gates
            #[derive(Copy, Clone, Eq, PartialEq, Hash)]
            $vis enum [<$name GateType>] {
                $($gate),+
            }

            /// Each gate in `mockturtle` has a name, fanin
            impl $crate::GateType for [<$name GateType>] {
                type Node = $name;
                const VARIANTS: &'static [Self] = &[
                    $(Self::$gate),+
                    $(Self::$gate_nary),*
                ];

                fn name(&self) -> &'static str {
                    match self {
                        $(Self::$gate => stringify!([<$gate:snake:lower>])),+
                        $(Self::$gate_nary => stringify!([<$gate_nary:snake:lower>])),*
                    }
                }

                fn fanin(&self) -> u8 {
                    match self {
                        $(Self::$gate => $fanin),+
                        $(Self::$gate_nary => $fanin_nary),*
                    }
                }

                fn mockturtle_create(&self) -> &'static str {
                    match self {
                        $(Self::$gate => concat!("create_", $gate_str)),+
                        $(Self::$gate_nary => concat!("create_nary_", $gate_nary_str)),*
                    }
                }

                fn mockturtle_is(&self) -> &'static str {
                    match self {
                        $(Self::$gate => concat!("is_", $gate_str)),+
                        $(Self::$gate_nary => concat!("is_nary", $gate_nary_str)),*
                    }
                }
            }

            /// FFI for calling network-specific functions in mockturtle
            #[repr(C)]
            $vis struct [<$name ReceiverFFI>]<R> {
                data: *mut $crate::libc::c_void,
                create_input: extern "C" fn(*mut $crate::libc::c_void, name: u64) -> $crate::Signal,
                create_constant: extern "C" fn (*mut $crate::libc::c_void, value: bool) -> $crate::Signal,
                $([<create_ $gate:snake:lower>]: $crate::seq_macro::seq!(N in 1..=$fanin {
                     extern "C" fn(*mut $crate::libc::c_void, #(input~N: $crate::Signal,)*) -> $crate::Signal
                })),+,
                $([<create_nary_ $gate_nary:snake:lower>]: $crate::seq_macro::seq!(N in 1..=$fanin_nary {
                     extern "C" fn(*mut $crate::libc::c_void, #(input~N: $crate::Signal,)*) -> $crate::Signal
                })),*
                done: extern "C" fn (*mut $crate::libc::c_void, outputs: *const $crate::Signal, outputs_size: usize) -> R,
            }

            impl<R> $crate::ReceiverFFI for [<$name ReceiverFFI>]<R> {
                fn new<Recv>(receiver: Recv) -> Self
                where
                    Recv: $crate::Receiver<Node = $name, Result = R> + 'static
                {
                    let data = Box::into_raw(Box::new(receiver));
                    Self {
                        data: data as *mut $crate::libc::c_void,
                        create_input: Self::create_input::<Recv>,
                        create_constant: Self::create_constant::<Recv>,
                        $([<create_ $gate:snake:lower>]: Self::[<create_ $gate:snake:lower>]::<Recv>),+,
                        $([<create_nary_ $gate_nary:snake:lower>]: Self::[<create_nary_ $gate_nary:snake:lower>]::<Recv>),*
                        done: Self::done::<Recv>,
                    }
                }
            }

            impl<R> $crate::Receiver for [<$name ReceiverFFI>]<R> {
                type Node = $name;
                type Result = R;

                /// Wrapper around `.create_*` methods offered for network by mockturtle (and
                /// exposed via `<$name ReceiverFFI>`
                fn create_node(&mut self, node: $name) -> $crate::Signal {
                    match node {
                        $name::Input(name) => (self.create_input)(self.data, name),
                        $name::False => (self.create_constant)(self.data, false),
                        $($name::$gate(ids) => {
                            $crate::seq_macro::seq!(N in 0..$fanin {
                                (self.[<create_ $gate:snake:lower>])(self.data, #(ids[N],)*)
                            })
                        }),+
                        $($name::$gate_nary(ids) => {
                            $crate::seq_macro::seq!(N in 0..$fanin_nary {
                                (self.[<create_ $gate_nary:snake:lower>])(self.data, #(ids[N],)*)
                            })
                        }),*
                    }
                }

                fn done(self, outputs: &[$crate::Signal]) -> R {
                    (self.done)(self.data, outputs.as_ptr(), outputs.len())
                }
            }

            /// FFI for network-specific functions provided by `mockturtle`
            impl<R> [<$name ReceiverFFI>]<R> {
                extern "C" fn create_input<Recv>(
                    data: *mut $crate::libc::c_void,
                    name: u64
                ) -> $crate::Signal
                where
                    Recv: $crate::Receiver<Node = $name, Result = R> + 'static
                {
                    unsafe { &mut *(data as *mut Recv) }.create_node($name::Input(name))
                }

                extern "C" fn create_constant<Recv>(
                    data: *mut $crate::libc::c_void,
                    value: bool
                ) -> $crate::Signal
                where
                    Recv: $crate::Receiver<Node = $name, Result = R> + 'static
                {
                    unsafe { &mut *(data as *mut Recv) }
                        .create_node($name::False)
                        .maybe_invert(value)
                }

                // NEXT: TODO !
                $($crate::seq_macro::seq!(N in 1..=$fanin {
                    pub extern "C" fn [<create_ $gate:snake:lower>]<Recv>(
                        data: *mut $crate::libc::c_void
                        #(, input~N: $crate::Signal)*
                    ) -> $crate::Signal
                    where
                        Recv: $crate::Receiver<Node = $name, Result = R> + 'static
                    {
                        unsafe { &mut *(data as *mut Recv) }.create_node($name::$gate([#(input~N,)*]))
                    }
                });)+

                $($crate::seq_macro::seq!(N in 1..=$fanin_nary {
                    pub extern "C" fn [<create_ $gate_nary:snake:lower>]<Recv>(
                        data: *mut $crate::libc::c_void
                        #(, input~N: $crate::Signal)*
                    ) -> $crate::Signal
                    where
                        Recv: $crate::Receiver<Node = $name, Result = R> + 'static
                    {
                        unsafe { &mut *(data as *mut Recv) }.create_node($name::$gate_nary([#(input~N,)*]))
                    }
                });)*

                extern "C" fn done<Recv>(
                    data: *mut $crate::libc::c_void,
                    outputs: *const $crate::Signal,
                    outputs_size: usize,
                ) -> R
                where
                    Recv: $crate::Receiver<Node = $name, Result = R> + 'static
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
