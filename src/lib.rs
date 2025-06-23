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
        gates {
            "maj" = Maj(3)
        }
    }
}

define_network! {
    pub enum "aig" = Aig {
        gates {
            "and" = And(2)
        }
    }
}

define_network! {
    pub enum "aoig" = Aoig {
        gates {
            "and" = And(2),
            "or" = Or(2)
        }
        nary_gates {
            "and2" = And2(2),
            "and4" = And4(4),
            "and8" = And8(8),
            "and16" = And16(16),
            "and32" = And32(32),
            "or2" = Or2(2),
            "or4" = Or4(4),
            "or8" = Or8(8),
            "or16" = Or16(16),
            "or32" = Or32(32)
        }
    }
}

define_network! {
    pub enum "xag" = Xag {
        gates {
            "and" = And(2),
            "xor" = Xor(2)
        }
    }
}

define_network! {
    pub enum "xmg" = Xmg {
        gates {
            "xor" = Xor(2),
            "maj" = Maj(3)
        }
    }
}
