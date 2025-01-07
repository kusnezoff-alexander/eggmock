#include <eggmock/generated.h>
extern "C" {
extern eggmock_mig_rewrite test_rewrite();
}

#include <eggmock/eggmock.h>

int main() {
  auto rewrite = test_rewrite();
  rewrite.transfer.create_const( rewrite.data, true );
  rewrite.transfer.create_not( rewrite.data, false );
  rewrite.transfer.create
}