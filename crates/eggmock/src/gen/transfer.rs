use crate::GateType;
use super::*;

impl<T: TransferFFI> StructName for T {
    fn struct_name() -> String {
        format!("eggmock_{}_transfer", T::Network::TYPENAME)
    }
}

impl<T: TransferFFI> CFFI for T {
    fn c_ffi() -> String {
        let mut additional_fields = "".to_string();
        for gate in T::Network::GATE_TYPES {
            additional_fields += format!(
                "  uint64_t ( *create_{} )( void* data {});",
                gate.name(),
                ", uint64_t".repeat(gate.fanin() as usize)
            ).as_str();
        }
        formatdoc!(
            r#"
                struct {} {{
                  uint64_t ( *create_symbol )( void* data, uint64_t name );
                  uint64_t ( *create_const )( void* data, bool value );
                  uint64_t ( *create_not )( void* data, uint64_t id );
                {additional_fields}
                }};
            "#,
            Self::struct_name()
        )
    }
}
