
#[macro_export]
macro_rules! define_network {
    ($(#[$meta:meta])* $vis:vis enum $name_str:literal = $name:ident {
        $($variant_str:literal = $variant:ident($count:literal)),+
    }) => {
        $crate::paste::paste! {
            $crate::egg::define_language! {
                $(#[$meta])*
                $vis enum $name {
                    Symbol(u64),
                    $($variant_str = $variant([$crate::egg::Id;$count])),+,
                }
            }

            impl $crate::Node for $name {
                const TYPENAME: &'static str = $name_str;
                type Variants = $crate::paste::paste!([<$name Variant>]);

                const TYPES: &'static [Self::Variants] = &[
                    $([<$name Variant>]::$variant),+
                ];
            }

            #[derive(Copy, Clone, Eq, PartialEq, Hash)]
            $vis enum [<$name Variant>] {
                $($variant),+
            }

            impl $crate::NodeType for [<$name Variant>] {
                type Language = $name;

                fn name(&self) -> &'static str {
                    match self {
                        $(Self::$variant => stringify!([<$variant:snake:lower>])),+
                    }
                }

                fn fanin(&self) -> u8 {
                    match self {
                        $(Self::$variant => $count),+
                    }
                }
            }

            mod [<_eggmock_ $name:snake:lower _ffi>] {
                use super::*;
                pub struct Data<R: $crate::Rewriter<Network = $name>> {
                    graph: Option<$crate::egg::EGraph<$name, R::Analysis>>,
                    rewriter: R,
                }

                impl<R: $crate::Rewriter<Network = $name>> Data<R> {
                    pub fn new(mut rewriter: R) -> Self {
                        Self {
                            graph: Some($crate::egg::EGraph::new(rewriter.create_analysis())),
                            rewriter
                        }
                    }
                }

                pub extern "C" fn add_symbol<R: $crate::Rewriter<Network = $name>>(
                    data: *mut $crate::libc::c_void,
                    name: u64
                ) -> u64 {
                    let graph = &mut unsafe { &mut *(data as *mut Data<R>) }.graph;
                    usize::from(graph.as_mut().unwrap().add($name::Symbol(name))) as u64
                }
                $($crate::seq_macro::seq!(N in 1..=$count {
                    pub extern "C" fn [<add_ $variant:snake:lower>]<R>(
                        data: *mut $crate::libc::c_void,
                        #(id~N: u64,)*
                    ) -> u64
                    where
                        R: $crate::Rewriter<Network = $name>
                    {
                        let graph = &mut unsafe { &mut *(data as *mut Data<R>) }.graph;
                        usize::from(
                            graph
                                .as_mut()
                                .unwrap()
                                .add($name::$variant([#($crate::egg::Id::from(id~N as usize),)*]))
                        ) as u64
                    }
                });)+
                pub extern "C" fn rewrite<R: $crate::Rewriter<Network = $name>>(
                    data: *mut $crate::libc::c_void,
                    roots_size: $crate::libc::size_t,
                    roots: *const u64,
                    callback: [<$name RewriterCallback>]
                ) {
                    let roots = unsafe { std::slice::from_raw_parts(roots, roots_size as usize) };
                    let roots = roots.iter().map(|root| $crate::egg::Id::from(*root as usize));
                    let data = unsafe { &mut *(data as *mut Data<R>) };
                    let result = data.rewriter.rewrite(data.graph.take().unwrap(), roots);
                    let mut node_ids = std::collections::HashMap::new();
                    for (id, node) in result.expr.items() {
                        let node_id = match node {
                            $name::Symbol(name) => callback.add_symbol(*name),
                            $(
                                #[allow(unused_variables)]
                                $name::$variant(children) => {
                                    let mapped_children = $crate::seq_macro::seq!(N in 0..$count {
                                        [ #(*node_ids.get(&children[N]).unwrap(),)* ]
                                    });
                                    callback.[<add_ $variant:snake:lower>](mapped_children)
                                }
                            ),+
                        };
                        node_ids.insert(id, node_id);
                    }
                    let roots = Vec::from_iter(
                        result.roots.iter().map(|root| *node_ids.get(&root).unwrap())
                    );
                    callback.mark_roots(roots.as_slice());
                    data.graph = Some($crate::egg::EGraph::new(data.rewriter.create_analysis()));
                }

                pub extern "C" fn free<R: $crate::Rewriter<Network = $name>>(
                    data: *mut $crate::libc::c_void
                ) {
                    unsafe { let _ = Box::from_raw(data as *mut Data<R>); }
                }
            }

            #[repr(C)]
            $vis struct [<$name RewriterFFI>] {
                data: *mut $crate::libc::c_void,
                add_symbol: extern "C" fn(*mut $crate::libc::c_void, name: u64) -> u64,
                $([<add_ $variant:snake:lower>]: $crate::seq_macro::seq!(N in 1..=$count {
                     extern "C" fn(*mut $crate::libc::c_void, #(id~N: u64,)*) -> u64
                })),+,
                rewrite: extern "C" fn(
                    *mut $crate::libc::c_void,
                    roots_size: $crate::libc::size_t,
                    roots: *const u64,
                    callback: [<$name RewriterCallback>]
                ),
                free: extern "C" fn (*mut $crate::libc::c_void)
            }

            impl [<$name RewriterFFI>] {
                pub fn new<R: Rewriter<Network = $name>>(rewriter: R) -> Self {
                    type Data<R> = [<_eggmock_ $name:snake:lower _ffi>]::Data::<R>;
                    let b = Box::new(Data::<R>::new(rewriter));
                    let ptr = Box::into_raw(b);
                    Self {
                        data: ptr as *mut $crate::libc::c_void,
                        add_symbol: [<_eggmock_ $name:snake:lower _ffi>]::add_symbol::<R>,
                        $(
                            [<add_ $variant:snake:lower>]: [<_eggmock_ $name:snake:lower _ffi>]
                                ::[<add_ $variant:snake:lower>]::<R>
                        ),+,
                        rewrite: [<_eggmock_ $name:snake:lower _ffi>]::rewrite::<R>,
                        free: [<_eggmock_ $name:snake:lower _ffi>]::free::<R>
                    }
                }
            }

            #[derive(Copy, Clone)]
            #[repr(C)]
            $vis struct [<$name RewriterCallback>] {
                data: *mut $crate::libc::c_void,
                add_symbol: extern "C" fn(*mut $crate::libc::c_void, name: u64) -> u64,
                $([<add_ $variant:snake:lower>]: $crate::seq_macro::seq!(N in 1..=$count {
                     extern "C" fn(*mut $crate::libc::c_void, #(id~N: u64,)*) -> u64
                })),+,
                mark_roots: extern "C" fn(
                    *mut $crate::libc::c_void,
                    roots_size: $crate::libc::size_t,
                    roots: *const u64
                ),
            }

            impl [<$name RewriterCallback>] {
                pub fn add_symbol(&self, name: u64) -> u64 {
                    (self.add_symbol)(self.data, name)
                }
                $(
                    #[allow(unused_variables)]
                    pub fn [<add_ $variant:snake:lower>](&self, children: [u64;$count]) -> u64 {
                        $crate::seq_macro::seq!(N in 0..$count {
                            (self.[<add_ $variant:snake:lower>])(self.data, #(children[N],)*)
                        })
                    }
                )+
                pub fn mark_roots(&self, roots: &[u64]) {
                    (self.mark_roots)(self.data, roots.len(), roots.as_ptr())
                }
            }
        }
    };
}