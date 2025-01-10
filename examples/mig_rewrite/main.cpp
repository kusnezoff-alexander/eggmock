#include "eggmock.h"
#include <mockturtle/mockturtle.hpp>

extern "C"
{
  extern eggmock::mig_rewrite example_mig_rewrite();
}

int main()
{
  mockturtle::mig_network network;
  auto pi0 = network.create_pi();
  auto pi1 = network.create_pi();
  auto pi2 = network.create_pi();

  auto sig1 = network.create_or( pi0, pi1 );
  auto sig2 = network.create_and( network.create_not( sig1 ), pi2 );
  auto out = rewrite_mig( network, example_mig_rewrite() );
}
