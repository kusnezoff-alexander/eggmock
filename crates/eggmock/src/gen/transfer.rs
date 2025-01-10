use super::*;
use crate::GateType;

impl<T: TransferFFI> StructFFI for T {
    fn struct_name() -> String {
        format!("{}_transfer", T::Network::TYPENAME)
    }
    fn struct_definition() -> String {
        let mut additional_fields = "".to_string();
        for gate in T::Network::GATE_TYPES {
            additional_fields += format!(
                "  uint64_t ( *create_{} )( void* data, {} );\n",
                gate.name(),
                id_parameters(gate)
            )
            .as_str();
        }
        formatdoc!(
            r#"
            struct {}
            {{
              uint64_t ( *create_symbol )( void* data, uint64_t name );
              uint64_t ( *create_const )( void* data, bool value );
              uint64_t ( *create_not )( void* data, uint64_t id );
            {additional_fields}}};
            "#,
            Self::struct_name()
        )
    }
}

pub fn transfer_helper<N: Network>() -> String {
    let transfer_struct = <N::TransferFFI as StructFFI>::struct_name();
    let ntk = N::TYPENAME;
    let ntk_type = format!("mockturtle::{}", N::MOCKTURTLE_TYPENAME);
    let mut gate_cases = "".to_string();
    for gate in N::GATE_TYPES {
        let fanin = gate.fanin();
        let gate_name = gate.name();
        let mockturtle_is = gate.mockturtle_is();
        let mut fanins = "".to_string();
        for i in 0..fanin {
            fanins += ", fanins[";
            fanins += i.to_string().as_str();
            fanins += "]";
        }
        gate_cases += format!(
            "
  else if ( ntk.{mockturtle_is}( node ) )
  {{
    uint64_t fanins[{fanin}];
    ntk.foreach_fanin( node, [&]( {ntk_type}::signal const& fanin, uint32_t const index ) {{
      fanins[index] = transfer_{ntk}_signal_id( ntk, fanin, transfer_data, transfer );
    }});
    id = transfer.create_{gate_name}( transfer_data{fanins} );
  }}",
        )
        .as_str();
    }
    formatdoc!(
        r#"
        namespace _impl
        {{
        inline uint64_t transfer_{ntk}_signal_id ( {ntk_type} const& ntk, {ntk_type}::signal const& signal, void* transfer_data, {transfer_struct} const& transfer ) {{
          auto const node = ntk.get_node( signal );

          uint64_t id;
          if ( ntk.visited( node ) )
          {{
            id = ntk.value( node );
          }}
          else if ( ntk.is_pi( node ) )
          {{
            id = transfer.create_symbol( transfer_data, ntk.pi_index( node ) );
          }}
          else if ( ntk.is_constant ( node ) )
          {{
            id = transfer.create_const( transfer_data, ntk.constant_value( node ) );
          }}{gate_cases}
          else
          {{
            throw std::invalid_argument( "unexpected node type" );
          }}

          if ( ntk.is_complemented( signal ) )
          {{
            id = transfer.create_not( transfer_data, id );
          }}

          ntk.set_value( node, id );
          ntk.set_visited( node, true );
          return id;
        }}
        }}
        inline void transfer_{ntk} ( {ntk_type} const& ntk, void* transfer_data, {transfer_struct} const& transfer )
        {{
            ntk.clear_values();
            ntk.clear_visited();
            ntk.foreach_node( [&] ( auto node ) {{
                _impl::transfer_{ntk}_signal_id( ntk, ntk.make_signal( node ), transfer_data, transfer );
            }} );
        }}
        "#,
    )
}

pub fn receive_helper<N: Network>() -> String {
    let transfer_struct = <N::TransferFFI as StructFFI>::struct_name();
    let ntk = N::TYPENAME;
    let ntk_type = format!("mockturtle::{}", N::MOCKTURTLE_TYPENAME);

    let mut struct_initializers = format!(
        r#"      .create_symbol = _impl::receive_{ntk}_create_symbol,
      .create_const = _impl::receive_{ntk}_create_const,
      .create_not = _impl::receive_{ntk}_create_not,
"#
    );
    let mut impl_methods = formatdoc!(
        r#"
        inline uint64_t receive_{ntk}_create_symbol( void* data, uint64_t name )
        {{
          auto const ntk = static_cast<{ntk_type} *>( data );
          while ( ntk->num_pis() < name )
          {{
            ntk->create_pi();
          }}
          return ntk->make_signal( ntk->pi_at( name ) ).data;
        }}

        inline uint64_t receive_{ntk}_create_const( void* data, bool value )
        {{
          auto const ntk = static_cast<{ntk_type} *>( data );
          return ntk->get_constant( value ).data;
        }}

        inline uint64_t receive_{ntk}_create_not( void* data, uint64_t id )
        {{
          auto const ntk = static_cast<{ntk_type} *>( data );
          return ntk->create_not( {ntk_type}::signal( id ) ).data;
        }}
        "#
    );
    for gate in N::GATE_TYPES {
        let gate_name = gate.name();
        let id_signals = (1..=gate.fanin())
            .map(|id| format!("{ntk_type}::signal( id{id} )"))
            .fold("".to_string(), |acc, x| acc + ", " + x.as_str());
        struct_initializers +=
            format!("      .create_{gate_name} = _impl::receive_{ntk}_create_{gate_name},\n")
                .as_str();
        impl_methods += formatdoc!(
            r#"
            inline uint64_t receive_{ntk}_create_{gate_name}( void* data, {ids} )
            {{
              auto const ntk = static_cast<{ntk_type} *>( data );
              return ntk->{create}( {id_signals} ).data;
            }}
            "#,
            ids = id_parameters(gate),
            create = gate.mockturtle_create(),
            id_signals = &id_signals[2..]
        )
        .as_str();
    }

    formatdoc!(
        r#"
        namespace _impl
        {{
        {impl_methods}
        }}
        inline {transfer_struct} receive_{ntk}()
        {{
          return {{
        {struct_initializers}
          }};
        }}
        "#
    )
}

fn id_parameters<G: GateType>(gate: &G) -> String {
    let mut res = "".to_string();
    for i in 1..=gate.fanin() {
        if i != 1 {
            res += ", ";
        }
        res += "uint64_t id";
        res += i.to_string().as_str();
    }
    res
}
