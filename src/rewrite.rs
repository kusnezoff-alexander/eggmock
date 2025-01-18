use crate::{AsNetworkTransfer, Network, NetworkTransfer, TransferFFI};
use egg::{Analysis, CostFunction, EGraph, Extractor, Id, RecExpr};
use std::collections::HashMap;

pub trait Rewriter: Sized {
    type Network: Network;
    type Analysis: Analysis<<Self::Network as Network>::Language>;

    fn create_analysis(&mut self) -> Self::Analysis;
    fn rewrite(
        &mut self,
        egraph: EGraph<<Self::Network as Network>::Language, Self::Analysis>,
        roots: impl Iterator<Item = Id>,
    ) -> RewriterResult<Self::Network>;
}

#[derive(Debug)]
pub struct RewriterResult<N: Network> {
    pub expr: RecExpr<N::Language>,
    pub roots: Vec<Id>,
}

impl<N: Network> RewriterResult<N> {
    pub fn extract_greedy<A: Analysis<N::Language>, CF: CostFunction<N::Language>>(
        graph: &EGraph<N::Language, A>,
        cost: CF,
        roots: impl IntoIterator<Item = Id>,
    ) -> Self {
        let extractor = Extractor::new(graph, cost);
        let mut expr = Vec::new();
        let mut expr_roots = vec![];
        for root in roots {
            expr.extend(extractor.find_best(root).1);
            expr_roots.push(Id::from(expr.len() - 1));
        }
        Self {
            expr: RecExpr::from(expr),
            roots: expr_roots
        }
    }
}

struct RewriteData<R: Rewriter> {
    graph: EGraph<<R::Network as Network>::Language, R::Analysis>,
    rewriter: R,
}

#[repr(C)]
pub struct RewriteFFI<N: Network> {
    data: *mut libc::c_void,
    transfer: N::TransferFFI,
    rewrite: extern "C" fn(
        *mut libc::c_void,
        roots: *mut u64,
        roots_size: libc::size_t,
        callback: RewriteCallback<N>,
    ),
    free: extern "C" fn(*mut libc::c_void),
}

#[repr(C)]
pub struct RewriteCallback<N: Network> {
    data: *mut libc::c_void,
    transfer: N::TransferFFI,
}

impl<R> AsNetworkTransfer<R::Network> for RewriteData<R>
where
    R: Rewriter,
    EGraph<<R::Network as Network>::Language, R::Analysis>: NetworkTransfer<R::Network>,
{
    fn as_transfer(&mut self) -> &mut impl NetworkTransfer<R::Network> {
        &mut self.graph
    }
}

impl<N> RewriteFFI<N>
where
    N: Network,
{
    pub fn new<R>(mut rewriter: R) -> Self
    where
        R: Rewriter<Network = N>,
        EGraph<<R::Network as Network>::Language, R::Analysis>: NetworkTransfer<R::Network>,
    {
        let data = RewriteData::<R> {
            graph: EGraph::new(rewriter.create_analysis()),
            rewriter,
        };
        Self {
            data: Box::into_raw(Box::new(data)) as *mut libc::c_void,
            transfer: <R::Network as Network>::TransferFFI::new::<RewriteData<R>>(),
            rewrite: Self::rewrite::<R>,
            free: Self::free::<R>,
        }
    }

    extern "C" fn rewrite<R: Rewriter<Network = N>>(
        data: *mut libc::c_void,
        roots: *mut u64,
        roots_size: libc::size_t,
        callback: RewriteCallback<N>,
    ) {
        let data = unsafe { &mut *(data as *mut RewriteData<R>) };
        let roots = if roots == std::ptr::null_mut() {
            [].as_mut_slice()
        } else {
            unsafe { std::slice::from_raw_parts_mut(roots, roots_size) }
        };

        let mut graph = EGraph::new(data.rewriter.create_analysis());
        std::mem::swap(&mut data.graph, &mut graph);
        let result = data.rewriter.rewrite(
            graph,
            roots.iter().map(|root| egg::Id::from(*root as usize)),
        );
        let mut map = HashMap::new();
        for (id, node) in result.expr.items() {
            let node = N::from(node.clone());
            let node_id = callback
                .transfer
                .create(callback.data, node.map_children(|id| map[&id]));
            map.insert(usize::from(id) as u64, node_id);
        }

        for (i, root) in roots.iter_mut().enumerate() {
            *root = map[&(usize::from(result.roots[i]) as u64)]
        }
    }

    extern "C" fn free<R: Rewriter>(data: *mut libc::c_void) {
        unsafe {
            let _ = Box::from_raw(data as *mut RewriteData<R>);
        }
    }
}
