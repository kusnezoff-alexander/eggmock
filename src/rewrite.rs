use crate::{Network, Receiver, ReceiverFFI};

pub trait Rewriter {
    type Network: Network;
    type Intermediate;
    type Receiver: Receiver<Network = Self::Network, Result = Self::Intermediate>;

    fn create_receiver(&mut self) -> Self::Receiver;
    fn rewrite(
        self,
        input: Self::Intermediate,
        output: impl Receiver<Network = Self::Network, Result = ()>,
    );
}

#[repr(C)]
pub struct RewriterFFI<N: Network> {
    data: *mut libc::c_void,
    rewrite: extern "C" fn(*mut libc::c_void, N::ReceiverFFI<()>),
}

impl<N: Network> RewriterFFI<N> {
    pub fn new<R>(mut rewriter: R) -> N::ReceiverFFI<RewriterFFI<N>>
    where
        R: Rewriter<Network = N> + 'static,
        R::Intermediate: 'static
    {
        N::ReceiverFFI::new(rewriter.create_receiver().map(|result| {
            let data = Box::into_raw(Box::new((rewriter, result)));
            RewriterFFI {
                data: data as *mut libc::c_void,
                rewrite: Self::rewrite::<R>,
            }
        }))
    }

    extern "C" fn rewrite<R: Rewriter<Network = N>>(
        data: *mut libc::c_void,
        callback: N::ReceiverFFI<()>,
    ) {
        let data = unsafe { Box::from_raw(data as *mut (R, R::Intermediate)) };
        data.0.rewrite(data.1, callback)
    }
}
