mod gen;
mod macros;
mod network;
mod rewrite;
mod transfer;
mod egg_impls;

pub use egg;
pub use libc;
pub use paste;
pub use seq_macro;

pub use network::*;
pub use rewrite::*;
pub use transfer::*;
pub use egg_impls::*;

pub use gen::*;

define_network! {
    pub enum "mig" = Mig {
        "maj" = Maj(3)
    }
}

define_network! {
    pub enum "aig" = Aig {
        "and" = And(2)
    }
}

define_network! {
    pub enum "xag" = Xag {
        "and" = And(2),
        "xor" = Xor(2)
    }
}

define_network! {
    pub enum "xmg" = Xmg {
        "xor" = Xor(2),
        "maj" = Maj(3)
    }
}
