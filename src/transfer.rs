use std::marker::PhantomData;

use super::{Node, Signal};

/// A type that can receive nodes of a logic network and produce some result from it.
pub trait Receiver: Sized {
    type Node: Node;
    type Result;

    /// Creates the given signal. Returns the id of the newly created signal.
    fn create_node(&mut self, node: Self::Node) -> Signal;
    /// Creates the result from the previously transferred nodes where `outputs` contains the output
    /// signals.
    fn done(self, outputs: &[Signal]) -> Self::Result;
    /// Maps the result of this Receiver using the given function.
    fn map<Res2, F>(self, map: F) -> impl Receiver<Node = Self::Node, Result = Res2>
    where
        F: FnOnce(Self::Result) -> Res2,
    {
        MappedReceiver {
            original: self,
            map,
        }
    }
    fn adapt<From: Node, F>(self, adapter: F) -> AdaptedReceiver<From, Self, F>
    where
        F: FnMut(From) -> Self::Node,
    {
        AdaptedReceiver {
            _from: PhantomData,
            to: self,
            adapter,
        }
    }
}

pub trait ReceiverFFI: Receiver {
    fn new<R>(receiver: R) -> Self
    where
        R: Receiver<Node = Self::Node, Result = Self::Result> + 'static;
}

struct MappedReceiver<Original, Function> {
    original: Original,
    map: Function,
}

impl<O, R, F> Receiver for MappedReceiver<O, F>
where
    O: Receiver,
    F: FnOnce(O::Result) -> R,
{
    type Node = O::Node;
    type Result = R;
    fn create_node(&mut self, node: Self::Node) -> Signal {
        self.original.create_node(node)
    }
    fn done(self, outputs: &[Signal]) -> Self::Result {
        (self.map)(self.original.done(outputs))
    }
}

pub struct AdaptedReceiver<From, To, F> {
    _from: PhantomData<fn(From) -> ()>,
    to: To,
    adapter: F,
}

impl<From, To, F> Receiver for AdaptedReceiver<From, To, F>
where
    From: Node,
    To: Receiver,
    F: FnMut(From) -> To::Node,
{
    type Node = From;
    type Result = To::Result;

    fn create_node(&mut self, node: Self::Node) -> Signal {
        self.to.create_node((self.adapter)(node))
    }

    fn done(self, outputs: &[Signal]) -> Self::Result {
        self.to.done(outputs)
    }
}
