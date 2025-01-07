#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum Mig<> { Symbol(u64), True([::eggmock::egg::Id; 0]), False([::eggmock::egg::Id; 0]), Not([::eggmock::egg::Id; 1]), Maj([::eggmock::egg::Id; 3]) }
impl<> ::egg::Language for Mig<>
where
{
    type Discriminant = std::mem::Discriminant<Self>;

    #[inline(always)]
    fn discriminant(&self) -> Self::Discriminant {
        std::mem::discriminant(self)
    }

    #[inline(always)]
    fn matches(&self, other: &Self) -> bool {
        ::std::mem::discriminant(self) == ::std::mem::discriminant(other) &&
            match (self, other) {
                (Mig::Symbol(data1), Mig::Symbol(data2)) => data1 == data2,
                (Mig::True(l), Mig::True(r)) => ::egg::LanguageChildren::len(l) == ::egg::LanguageChildren::len(r),
                (Mig::False(l), Mig::False(r)) => ::egg::LanguageChildren::len(l) == ::egg::LanguageChildren::len(r),
                (Mig::Not(l), Mig::Not(r)) => ::egg::LanguageChildren::len(l) == ::egg::LanguageChildren::len(r),
                (Mig::Maj(l), Mig::Maj(r)) => ::egg::LanguageChildren::len(l) == ::egg::LanguageChildren::len(r),
                _ => false
            }
    }

    fn children(&self) -> &[::egg::Id] {
        match self {
            Mig::Symbol(_data) => &[],
            Mig::True(ids) => ::egg::LanguageChildren::as_slice(ids),
            Mig::False(ids) => ::egg::LanguageChildren::as_slice(ids),
            Mig::Not(ids) => ::egg::LanguageChildren::as_slice(ids),
            Mig::Maj(ids) => ::egg::LanguageChildren::as_slice(ids),
        }
    }
    fn children_mut(&mut self) -> &mut [::egg::Id] {
        match self {
            Mig::Symbol(_data) => &mut [],
            Mig::True(ids) => ::egg::LanguageChildren::as_mut_slice(ids),
            Mig::False(ids) => ::egg::LanguageChildren::as_mut_slice(ids),
            Mig::Not(ids) => ::egg::LanguageChildren::as_mut_slice(ids),
            Mig::Maj(ids) => ::egg::LanguageChildren::as_mut_slice(ids),
        }
    }
}
impl<> ::std::fmt::Display for Mig<>
where
{
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match (self, f) {
            (Mig::Symbol(data), f) => ::std::fmt::Display::fmt(data, f),
            (Mig::True(..), f) => f.write_str("t"),
            (Mig::False(..), f) => f.write_str("f"),
            (Mig::Not(..), f) => f.write_str("!"),
            (Mig::Maj(..), f) => f.write_str("m"),
        }
    }
}
impl<> ::egg::FromOp for Mig<>
where
{
    type Error = ::egg::FromOpError;

    fn from_op(op: &str, children: ::std::vec::Vec<::egg::Id>) -> ::std::result::Result<Self, Self::Error> {
        match (op, children) {
            (op, children)   if op.parse::<u64>().is_ok() && children.is_empty() => Ok(Mig::Symbol(op.parse().unwrap())),
            (op, children)   if op == "t" && <[::eggmock::egg::Id; 0] as ::egg::LanguageChildren>::can_be_length(children.len()) => {
                let children = <[::eggmock::egg::Id; 0] as ::egg::LanguageChildren>::from_vec(children);
                Ok(Mig::True(children))
            }
            (op, children)   if op == "f" && <[::eggmock::egg::Id; 0] as ::egg::LanguageChildren>::can_be_length(children.len()) => {
                let children = <[::eggmock::egg::Id; 0] as ::egg::LanguageChildren>::from_vec(children);
                Ok(Mig::False(children))
            }
            (op, children)   if op == "!" && <[::eggmock::egg::Id; 1] as ::egg::LanguageChildren>::can_be_length(children.len()) => {
                let children = <[::eggmock::egg::Id; 1] as ::egg::LanguageChildren>::from_vec(children);
                Ok(Mig::Not(children))
            }
            (op, children)   if op == "m" && <[::eggmock::egg::Id; 3] as ::egg::LanguageChildren>::can_be_length(children.len()) => {
                let children = <[::eggmock::egg::Id; 3] as ::egg::LanguageChildren>::from_vec(children);
                Ok(Mig::Maj(children))
            }

            (op, children) => Err(::egg::FromOpError::new(op, children)),
        }
    }
}
impl ::eggmock::NetworkLanguage for Mig {
    const TYPENAME: &'static str = "mig";
    type Variants = MigVariant;
}
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum MigVariant {
    Symbol,
    True,
    False,
    Not,
    Maj,
}
impl ::eggmock::NetworkLanguageVariant for MigVariant {
    type Language = Mig;

    const VARIANTS: &'static [Self] = &[
        Self::Symbol,
        Self::True, Self::False, Self::Not, Self::Maj
    ];

    fn name(&self) -> &'static str {
        match self {
            Self::Symbol => "symbol",
            Self::True => "true",
            Self::False => "false",
            Self::Not => "not",
            Self::Maj => "maj"
        }
    }

    fn fanin(&self) -> u8 {
        match self {
            Self::Symbol => 0,
            Self::True => 0,
            Self::False => 0,
            Self::Not => 1,
            Self::Maj => 3
        }
    }
}
mod _eggmock_mig_ffi {
    use super::*;
    pub struct Data<R: ::eggmock::Rewriter<Network=Mig>> {
        graph: Option<::eggmock::egg::EGraph<Mig, R::Analysis>>,
        rewriter: R,
    }

    impl<R: ::eggmock::Rewriter<Network=Mig>> Data<R> {
        pub fn new(mut rewriter: R) -> Self {
            Self {
                graph: Some(::eggmock::egg::EGraph::new(rewriter.create_analysis())),
                rewriter,
            }
        }
    }

    pub extern "C" fn add_symbol<R: ::eggmock::Rewriter<Network=Mig>>(
        data: *mut ::eggmock::libc::c_void,
        name: u64,
    ) -> u64 {
        let graph = &mut unsafe { &mut *(data as *mut Data<R>) }.graph;
        usize::from(graph.as_mut().unwrap().add(Mig::Symbol(name))) as u64
    }
    pub extern "C" fn add_true<R>(
        data: *mut ::eggmock::libc::c_void,
    ) -> u64
    where
        R: ::eggmock::Rewriter<Network=Mig>,
    {
        let graph = &mut unsafe { &mut *(data as *mut Data<R>) }.graph;
        usize::from(
            graph
                .as_mut()
                .unwrap()
                .add(Mig::True([]))
        ) as u64
    }
    pub extern "C" fn add_false<R>(
        data: *mut ::eggmock::libc::c_void,
    ) -> u64
    where
        R: ::eggmock::Rewriter<Network=Mig>,
    {
        let graph = &mut unsafe { &mut *(data as *mut Data<R>) }.graph;
        usize::from(
            graph
                .as_mut()
                .unwrap()
                .add(Mig::False([]))
        ) as u64
    }
    pub extern "C" fn add_not<R>(
        data: *mut ::eggmock::libc::c_void,
        id1: u64, ) -> u64
    where
        R: ::eggmock::Rewriter<Network=Mig>,
    {
        let graph = &mut unsafe { &mut *(data as *mut Data<R>) }.graph;
        usize::from(
            graph
                .as_mut()
                .unwrap()
                .add(Mig::Not([::eggmock::egg::Id::from(id1 as usize), ]))
        ) as u64
    }
    pub extern "C" fn add_maj<R>(
        data: *mut ::eggmock::libc::c_void,
        id1: u64, id2: u64, id3: u64, ) -> u64
    where
        R: ::eggmock::Rewriter<Network=Mig>,
    {
        let graph = &mut unsafe { &mut *(data as *mut Data<R>) }.graph;
        usize::from(
            graph
                .as_mut()
                .unwrap()
                .add(Mig::Maj([::eggmock::egg::Id::from(id1 as usize), ::eggmock::egg::Id::from(id2 as usize), ::eggmock::egg::Id::from(id3 as usize), ]))
        ) as u64
    }
    pub extern "C" fn rewrite<R: ::eggmock::Rewriter<Network=Mig>>(
        data: *mut ::eggmock::libc::c_void,
        roots_size: ::eggmock::libc::size_t,
        roots: *const u64,
        callback: MigRewriterCallback) {
        let roots = unsafe { std::slice::from_raw_parts(roots, roots_size as usize) };
        let roots = roots.iter().map(|root| ::eggmock::egg::Id::from(*root as usize));
        let data = unsafe { &mut *(data as *mut Data<R>) };
        let result = data.rewriter.rewrite(data.graph.take().unwrap(), roots);
        let mut node_ids = std::collections::HashMap::new();
        for (id, node) in result.expr.items() {
            let node_id = match node {
                Mig::Symbol(name) => callback.add_symbol(*name),
                #[allow(unused_variables)]
                Mig::True(children) => {
                    let mapped_children = [];
                    callback.add_true(mapped_children)
                }
                #[allow(unused_variables)]
                Mig::False(children) => {
                    let mapped_children = [];
                    callback.add_false(mapped_children)
                }
                #[allow(unused_variables)]
                Mig::Not(children) => {
                    let mapped_children = [*node_ids.get(&children[0]).unwrap(), ];
                    callback.add_not(mapped_children)
                }
                #[allow(unused_variables)]
                Mig::Maj(children) => {
                    let mapped_children = [*node_ids.get(&children[0]).unwrap(), *node_ids.get(&children[1]).unwrap(), *node_ids.get(&children[2]).unwrap(), ];
                    callback.add_maj(mapped_children)
                }
            };
            node_ids.insert(id, node_id);
        }
        let roots = Vec::from_iter(
            result.roots.iter().map(|root| *node_ids.get(&root).unwrap())
        );
        callback.mark_roots(roots.as_slice());
        data.graph = Some(::eggmock::egg::EGraph::new(data.rewriter.create_analysis()));
    }

    pub extern "C" fn free<R: ::eggmock::Rewriter<Network=Mig>>(
        data: *mut ::eggmock::libc::c_void
    ) {
        unsafe { let _ = Box::from_raw(data as *mut Data<R>); }
    }
}
#[repr(C)]
pub struct MigRewriterFFI {
    data: *mut ::eggmock::libc::c_void,
    add_symbol: extern "C" fn(*mut ::eggmock::libc::c_void, name: u64) -> u64,
    add_true: extern "C" fn(*mut ::eggmock::libc::c_void) -> u64,
    add_false: extern "C" fn(*mut ::eggmock::libc::c_void) -> u64,
    add_not: extern "C" fn(*mut ::eggmock::libc::c_void, id1: u64) -> u64,
    add_maj: extern "C" fn(*mut ::eggmock::libc::c_void, id1: u64, id2: u64, id3: u64) -> u64,
    rewrite: extern "C" fn(
        *mut ::eggmock::libc::c_void,
        roots_size: ::eggmock::libc::size_t,
        roots: *const u64,
        callback: MigRewriterCallback),
    free: extern "C" fn(*mut ::eggmock::libc::c_void),
}
impl MigRewriterFFI {
    pub fn new<R: Rewriter<Network=Mig>>(rewriter: R) -> Self {
        type Data<R> = _eggmock_mig_ffi::Data::<R>;
        let b = Box::new(Data::<R>::new(rewriter));
        let ptr = Box::into_raw(b);
        Self {
            data: ptr as *mut Data<R> as *mut ::eggmock::libc::c_void,
            add_symbol: _eggmock_mig_ffi::add_symbol::<R>,
            add_true: _eggmock_mig_ffi::add_true::<R>,
            add_false: _eggmock_mig_ffi::add_false::<R>,
            add_not: _eggmock_mig_ffi::add_not::<R>,
            add_maj: _eggmock_mig_ffi::add_maj::<R>,
            rewrite: _eggmock_mig_ffi::rewrite::<R>,
            free: _eggmock_mig_ffi::free::<R>,
        }
    }
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MigRewriterCallback {
    data: *mut ::eggmock::libc::c_void,
    add_symbol: extern "C" fn(*mut ::eggmock::libc::c_void, name: u64) -> u64,
    add_true: extern "C" fn(*mut ::eggmock::libc::c_void) -> u64,
    add_false: extern "C" fn(*mut ::eggmock::libc::c_void) -> u64,
    add_not: extern "C" fn(*mut ::eggmock::libc::c_void, id1: u64) -> u64,
    add_maj: extern "C" fn(*mut ::eggmock::libc::c_void, id1: u64, id2: u64, id3: u64) -> u64,
    mark_roots: extern "C" fn(
        *mut ::eggmock::libc::c_void,
        roots_size: ::eggmock::libc::size_t,
        roots: *const u64,
    ),
}
impl MigRewriterCallback {
    pub fn add_symbol(&self, name: u64) -> u64 {
        (self.add_symbol)(self.data, name)
    }
    #[allow(unused_variables)]
    pub fn add_true(&self, children: [u64; 0]) -> u64 {
        (self.add_true)(self.data)
    }
    #[allow(unused_variables)]
    pub fn add_false(&self, children: [u64; 0]) -> u64 {
        (self.add_false)(self.data)
    }
    #[allow(unused_variables)]
    pub fn add_not(&self, children: [u64; 1]) -> u64 {
        (self.add_not)(self.data, children[0])
    }
    #[allow(unused_variables)]
    pub fn add_maj(&self, children: [u64; 3]) -> u64 {
        (self.add_maj)(self.data, children[0], children[1], children[2])
    }
    pub fn mark_roots(&self, roots: &[u64]) {
        (self.mark_roots)(self.data, roots.len(), roots.as_ptr   (   )   )
    }
}