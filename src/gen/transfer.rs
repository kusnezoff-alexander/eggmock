use super::*;
use crate::{GateType, Network};

pub fn receiver_struct<N: Network>() -> String {
    let ntk = N::TYPENAME;
    let mut additional_fields = "".to_string();
    for gate in N::Gates::VARIANTS {
        additional_fields += format!(
            "\n  signal ( *create_{} )( void* data, {} );",
            gate.name(),
            signal_parameters(gate)
        )
        .as_str();
    }
    formatdoc!(
        r#"
        template<class result>
        struct {ntk}_receiver
        {{
          void* data;
          signal ( *create_input )( void* data, uint64_t name );
          signal ( *create_const )( void* data, bool value );{additional_fields}
          result ( *done )( void* data, signal const* roots, size_t roots_size );
        }};
        "#
    )
}

pub fn send_helper<N: Network>() -> String {
    let ntk = N::TYPENAME;
    let ntk_type = format!("mockturtle::{}", N::MOCKTURTLE_TYPENAME);
    let mut gate_cases = "".to_string();
    for gate in N::Gates::VARIANTS {
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
    signal fanins[{fanin}];
    ntk.foreach_fanin( node, [&]( {ntk_type}::signal const& fanin, uint32_t const index ) {{
      fanins[index] = send_{ntk}_signal( ntk, fanin, receiver );
    }} );
    dst_sig = receiver.create_{gate_name}( receiver.data{fanins} );
  }}",
        )
        .as_str();
    }
    formatdoc!(
        r#"
        namespace _impl
        {{
        template<class result>
        signal send_{ntk}_signal( {ntk_type} const& ntk, {ntk_type}::signal const& src_sig, {ntk}_receiver<result> const& receiver )
        {{
          auto const node = ntk.get_node( src_sig );

          signal dst_sig;
          if ( ntk.visited( node ) )
          {{
            dst_sig = signal( ntk.value( node ) );
          }}
          else if ( ntk.is_pi( node ) )
          {{
            dst_sig = receiver.create_input( receiver.data, ntk.pi_index( node ) );
          }}
          else if ( ntk.is_constant( node ) )
          {{
            dst_sig = receiver.create_const( receiver.data, ntk.constant_value( node ) );
          }}{gate_cases}
          else
          {{
            throw std::invalid_argument( "unexpected node type" );
          }}
          ntk.set_value( node, dst_sig._v );
          ntk.set_visited( node, true );

          if ( ntk.is_complemented( src_sig ) )
          {{
            dst_sig = dst_sig.complement();
          }}
          return dst_sig;
        }}
        }} // namespace _impl
        template<class result>
        result send_{ntk} ( {ntk_type} const& ntk, {ntk}_receiver<result> const& receiver )
        {{
          ntk.clear_values();
          ntk.clear_visited();
          ntk.foreach_node( [&] ( auto const& node ) {{
            _impl::send_{ntk}_signal( ntk, ntk.make_signal( node ), receiver );
          }} );

          std::vector<signal> outputs;
          outputs.reserve( ntk.num_pos() );
          ntk.foreach_po( [&] ( auto const& src_sig ) {{
            signal sig = _impl::send_{ntk}_signal( ntk, src_sig, receiver );
            outputs.emplace_back( sig );
          }} );
          return receiver.done( receiver.data, outputs.data(), outputs.size() );
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
        inline signal {ntk}_map_signal( {ntk_type} const& ntk, {ntk_type}::signal const& s ) {{
          return signal( ntk.node_to_index( ntk.get_node( s ) ), ntk.is_complemented( s ) );
        }}
        inline {ntk_type}::signal map_signal_{ntk}( {ntk_type}& ntk, signal s ) {{
          auto sig = ntk.make_signal( ntk.index_to_node ( s.id() ) );
          if ( s.is_complemented() ) {{
            sig = ntk.create_not( sig );
          }}
          return sig;
        }}

        inline signal receive_{ntk}_create_input( void* data, uint64_t name )
        {{
          auto const ntk = static_cast<{ntk_type}*>( data );
          while ( ntk->num_pis() <= name )
          {{
            ntk->create_pi();
          }}
          return signal( ntk->node_to_index( ntk->pi_at( name ) ), false );
        }}

        inline signal receive_{ntk}_create_const( void* data, bool value )
        {{
          auto const ntk = static_cast<{ntk_type}*>( data );
          return {ntk}_map_signal( *ntk, ntk->get_constant(value) ) ;
        }}

        inline void receive_{ntk}_done( void* data, signal const* roots, size_t roots_size )
        {{
          auto const ntk = static_cast<{ntk_type}*>( data );
          for ( size_t i = 0; i < roots_size; i++ )
          {{
            ntk->create_po( map_signal_{ntk}( *ntk, roots[i] ) );
          }}
        }}
        "#
    );
    for gate in N::Gates::VARIANTS {
        let gate_name = gate.name();
        let id_signals = (1..=gate.fanin())
            .map(|id| format!("map_signal_{ntk}( *ntk, input{id} )"))
            .fold("".to_string(), |acc, x| acc + ", " + x.as_str());
        struct_initializers +=
            format!("\n      .create_{gate_name} = _impl::receive_{ntk}_create_{gate_name},")
                .as_str();
        impl_methods += formatdoc!(
            r#"
            inline signal receive_{ntk}_create_{gate_name}( void* data, {ids} )
            {{
              auto const ntk = static_cast<{ntk_type}*>( data );
              return {ntk}_map_signal(
                  *ntk,
                  ntk->{create}( {id_signals} ) );
            }}
            "#,
            ids = signal_parameters(gate),
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
              .create_const = _impl::receive_{ntk}_create_const,{struct_initializers}
              .done = _impl::receive_{ntk}_done,
          }};
        }}
        "#
    )
}

fn signal_parameters<G: GateType>(gate: &G) -> String {
    let mut res = "".to_string();
    for i in 1..=gate.fanin() {
        if i != 1 {
            res += ", ";
        }
        res += "signal input";
        res += i.to_string().as_str();
    }
    res
}
