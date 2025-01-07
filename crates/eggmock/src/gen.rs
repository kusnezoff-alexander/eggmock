use crate::{Network, TransferFFI};
use indoc::formatdoc;

trait CFFI {
    fn c_ffi() -> String;
}

trait StructName {
    fn struct_name() -> String;
}

impl<T: TransferFFI> StructName for T {
    fn struct_name() -> String {
        format!("eggmock_{}_transfer", T::Network::TYPENAME)
    }
}

impl<T: TransferFFI> CFFI for T {
    fn c_ffi() -> String {
        let additional_fields = "".to_string();
        for gate in T::Network::GATE_TYPES {

        }
        formatdoc!(
            r#"
                struct {} {{
                    uint64_t ( *add_symbol )( void* data, uint64_t name );
                    uint64_t ( *add_const )( void* data, bool value );
                    uint64_t ( *add_not )( void* data, uint64_t id );
                    { additional_fields }
                }}
            "#,
            Self::struct_name()
        )
    }
}
