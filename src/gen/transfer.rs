use super::*;
use crate::GateType;

pub fn receiver_struct<N: Network>() -> String {
    let ntk = N::TYPENAME;
    let mut additional_fields = "".to_string();
    for gate in N::GateType::VARIANTS {
        additional_fields += format!(
            "\n  uint64_t ( *create_{} )( void* data, {} );",
            gate.name(),
            id_parameters(gate)
        )
        .as_str();
    }
    formatdoc!(
        r#"
        template<class result>
        struct {ntk}_receiver
        {{
          void* data;
          uint64_t ( *create_input )( void* data, uint64_t name );
          uint64_t ( *create_const )( void* data, bool value );
          uint64_t ( *create_not )( void* data, uint64_t id );{additional_fields}
          result ( *done )( void* data, uint64_t const* roots, size_t roots_size );
        }};
        "#
    )
}

pub fn send_helper<N: Network>() -> String {
    let ntk = N::TYPENAME;
    let ntk_type = format!("mockturtle::{}", N::MOCKTURTLE_TYPENAME);
    let mut gate_cases = "".to_string();
    for gate in N::GateType::VARIANTS {
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
      fanins[index] = send_{ntk}_signal_id( ntk, fanin, receiver );
    }} );
    id = receiver.create_{gate_name}( receiver.data{fanins} );
  }}",
        )
        .as_str();
    }
    formatdoc!(
        r#"
        namespace _impl
        {{
        template<class result>
        uint64_t send_{ntk}_signal_id( {ntk_type} const& ntk, {ntk_type}::signal const& signal, {ntk}_receiver<result> const& receiver )
        {{
          auto const node = ntk.get_node( signal );

          uint64_t id;
          if ( ntk.visited( node ) )
          {{
            id = ntk.value( node );
          }}
          else if ( ntk.is_pi( node ) )
          {{
            id = receiver.create_input( receiver.data, ntk.pi_index( node ) );
          }}
          else if ( ntk.is_constant( node ) )
          {{
            id = receiver.create_const( receiver.data, ntk.constant_value( node ) );
          }}{gate_cases}
          else
          {{
            throw std::invalid_argument( "unexpected node type" );
          }}

          if ( ntk.is_complemented( signal ) )
          {{
            id = receiver.create_not( receiver.data, id );
          }}

          ntk.set_value( node, id );
          ntk.set_visited( node, true );
          return id;
        }}
        }} // namespace _impl
        template<class result>
        result send_{ntk} ( {ntk_type} const& ntk, {ntk}_receiver<result> const& receiver )
        {{
          ntk.clear_values();
          ntk.clear_visited();
          ntk.foreach_node( [&] ( auto const& node ) {{
            _impl::send_{ntk}_signal_id( ntk, ntk.make_signal( node ), receiver );
          }} );

          std::vector<uint64_t> roots;
          roots.reserve( ntk.num_pos() );
          ntk.foreach_po( [&] ( auto const& signal ) {{
            uint64_t id = _impl::send_{ntk}_signal_id( ntk, signal, receiver );
            roots.emplace_back( id );
          }} );
          return receiver.done( receiver.data, roots.data(), roots.size() );
        }}
        "#,
    )
}

pub fn receive_helper<N: Network>() -> String {
    let ntk = N::TYPENAME;
    let ntk_type = format!("mockturtle::{}", N::MOCKTURTLE_TYPENAME);

    let mut struct_initializers = String::new();
    let mut impl_methods = formatdoc!(
        r#"
        inline uint64_t receive_{ntk}_create_input( void* data, uint64_t name )
        {{
          auto const ntk = static_cast<{ntk_type}*>( data );
          while ( ntk->num_pis() <= name )
          {{
            ntk->create_pi();
          }}
          return ntk->make_signal( ntk->pi_at( name ) ).data;
        }}

        inline uint64_t receive_{ntk}_create_const( void* data, bool value )
        {{
          auto const ntk = static_cast<{ntk_type}*>( data );
          return ntk->get_constant( value ).data;
        }}

        inline uint64_t receive_{ntk}_create_not( void* data, uint64_t id )
        {{
          auto const ntk = static_cast<{ntk_type}*>( data );
          return ntk->create_not( {ntk_type}::signal( id ) ).data;
        }}

        inline void receive_{ntk}_done( void* data, uint64_t const* roots, size_t roots_size )
        {{
          auto const ntk = static_cast<{ntk_type}*>( data );
          for ( size_t i = 0; i < roots_size; i++ )
          {{
            ntk->create_po( {ntk_type}::signal( roots[i] ) );
          }}
        }}
        "#
    );
    for gate in N::GateType::VARIANTS {
        let gate_name = gate.name();
        let id_signals = (1..=gate.fanin())
            .map(|id| format!("{ntk_type}::signal( id{id} )"))
            .fold("".to_string(), |acc, x| acc + ", " + x.as_str());
        struct_initializers +=
            format!("\n      .create_{gate_name} = _impl::receive_{ntk}_create_{gate_name},")
                .as_str();
        impl_methods += formatdoc!(
            r#"
            inline uint64_t receive_{ntk}_create_{gate_name}( void* data, {ids} )
            {{
              auto const ntk = static_cast<{ntk_type}*>( data );
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
        }} // namespace _impl
        inline {ntk}_receiver<void> receive_{ntk}({ntk_type}& ntk)
        {{
          return {{
              .data = &ntk,
              .create_input = _impl::receive_{ntk}_create_input,
              .create_const = _impl::receive_{ntk}_create_const,
              .create_not = _impl::receive_{ntk}_create_not,{struct_initializers}
              .done = _impl::receive_{ntk}_done,
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
