use crate::{AsNetworkTransfer, Network, NetworkTransfer, TransferFFI};
use egg::{Analysis, EGraph, Id, RecExpr};
use std::collections::HashMap;

pub trait Rewriter: Sized {
    type Network: Network;
    type Analysis: Analysis<<Self::Network as Network>::Language>;

    fn create_analysis(&mut self) -> Self::Analysis;
    fn rewrite(
        &mut self,
        egraph: EGraph<<Self::Network as Network>::Language, Self::Analysis>,
        roots: impl Iterator<Item = Id>,
    ) -> RewriterResult<Self>;
}

pub struct RewriterResult<R: Rewriter> {
    pub expr: RecExpr<<R::Network as Network>::Language>,
    pub roots: Vec<Id>,
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
}

#[repr(C)]
pub struct RewriteCallback<N: Network> {
    data: *mut libc::c_void,
    transfer: N::TransferFFI,
}

impl<N: Network> RewriteCallback<N> {}

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
        }
    }

    extern "C" fn rewrite<R: Rewriter<Network = N>>(
        data: *mut libc::c_void,
        roots: *mut u64,
        roots_size: libc::size_t,
        callback: RewriteCallback<N>,
    ) {
        let data = unsafe { &mut *(data as *mut RewriteData<R>) };
        let roots = unsafe { std::slice::from_raw_parts_mut(roots, roots_size) };

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
                .create(callback.data, node.map_ids(|id| map[&id]));
            map.insert(usize::from(id) as u64, node_id);
        }

        for root in roots {
            *root = map[root]
        }
    }
}
