use crate::{Network, Receiver, ReceiverFFI};

/// Allows rewriting of logic networks.
///
/// The rewriting process works as follows:
/// 1. create a [`Receiver`]
/// 2. send the network to the receiver
/// 3. perform the rewrite on the result of the [`Receiver`]
/// 4. send the result of the rewrite to an output [`Receiver`]
pub trait Rewriter {
    type Network: Network;
    type Intermediate;
    type Receiver: Receiver<Self::Network, Result = Self::Intermediate>;

    fn create_receiver(&mut self) -> Self::Receiver;
    fn rewrite(
        self,
        input: Self::Intermediate,
        output: impl Receiver<Self::Network, Result = ()>,
    );
}

/// A struct that contains a data pointer and a function pointing to the function that performs the
/// rewrite using the data.
///
/// Allocated memory is released after a call to the rewrite function.
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
