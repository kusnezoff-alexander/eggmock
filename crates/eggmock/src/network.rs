use egg::Language;

pub trait Network: From<Self::Language> {
    type GateType: GateType<Network = Self>;
    type Language: Language + From<Self>;
    type TransferFFI: TransferFFI<Network = Self>;

    const TYPENAME: &'static str;
    const GATE_TYPES: &'static [Self::GateType];

    fn map_ids(&self, map: impl Fn(u64) -> u64) -> Self;

    fn c_ffi() -> String {
        let typename = Self::TYPENAME;

        let mut fields = "uint64_t ( *add_symbol )( void* data, uint64_t name );".to_string();
        for variant in Self::GATE_TYPES {
            fields += "\n  uint64_t ( *add_";
            fields += variant.name();
            fields += " )( void* data";
            for i in 1..=variant.fanin() {
                fields += ", uint64_t id";
                fields += i.to_string().as_str();
            }
            fields += " );"
        }

        format!(
            r#"
#ifdef __cplusplus
extern "C" {{
#endif

#include <stdint.h>
#include <stddef.h>

struct eggmock_{typename}_ffi;
struct eggmock_{typename}_ffi_callback;

struct eggmock_{typename}_ffi
{{
  void* data;
  {fields}
  void ( *rewrite )( void* data, size_t roots_size, const uint64_t* roots, eggmock_{typename}_ffi_callback callback );
  void ( *free )( void* data );
}};

struct eggmock_{typename}_ffi_callback
{{
  void* data;
  {fields}
  void ( *mark_roots )( void* data, size_t roots_size, const uint64_t* roots );
}};


#ifdef __cplusplus
}}
#endif"#
        )
    }
}

pub trait GateType: Sized + 'static {
    type Network: Network<GateType = Self>;

    fn name(&self) -> &'static str;
    fn fanin(&self) -> u8;

    fn mockturtle_create(&self) -> &'static str;
    fn mockturtle_is(&self) -> &'static str;
}

pub trait NetworkTransfer<N: Network> {
    fn create(&mut self, node: N) -> u64;
}

pub trait TransferFFI {
    type Network: Network;

    fn new<T: AsNetworkTransfer<Self::Network>>() -> Self;
    fn create(&self, data: *mut libc::c_void, node: Self::Network) -> u64;
}

pub trait AsNetworkTransfer<N: Network> {
    fn as_transfer(&mut self) -> &mut impl NetworkTransfer<N>;
}
