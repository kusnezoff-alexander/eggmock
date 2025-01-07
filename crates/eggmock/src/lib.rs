mod macros;
mod network;
mod rewrite;
pub mod gen;

pub use egg;
pub use libc;
pub use paste;
pub use seq_macro;

pub use network::*;
pub use rewrite::*;

define_network! {
    pub enum Mig {
        "maj" = Maj(3, create_maj, is_maj)
    }
}

define_network! {
    pub enum Aig {
        "*" = And(3, create_and, is_and)
    }
}

define_network! {
    pub enum Xag {
        "*" = And(3, create_and, is_and),
        "xor" = Xor(3, create_xor, is_xor)
    }
}

define_network! {
    pub enum Xmg {
        "xor" = Xor(2, create_xor, is_xor),
        "maj" = Maj(3, create_maj, is_maj)
    }
}
