mod gen;
mod macros;
mod network;
mod rewrite;
mod transfer;

pub use egg;
pub use libc;
pub use paste;
pub use seq_macro;

pub use network::*;
pub use rewrite::*;
pub use transfer::*;

pub use gen::*;

define_network! {
    pub enum "mig" = Mig {
        "maj" = Maj(3, create_maj, is_maj)
    }
}

define_network! {
    pub enum "aig" = Aig {
        "*" = And(2, create_and, is_and)
    }
}

define_network! {
    pub enum "xag" = Xag {
        "*" = And(2, create_and, is_and),
        "xor" = Xor(2, create_xor, is_xor)
    }
}

define_network! {
    pub enum "xmg" = Xmg {
        "xor" = Xor(2, create_xor, is_xor),
        "maj" = Maj(3, create_maj, is_maj)
    }
}
