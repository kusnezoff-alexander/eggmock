#include <iostream>

#include <cstdint>
#include <mockturtle/mockturtle.hpp>

#ifdef __cplusplus
extern "C"
{
#endif

#include <stdint.h>

  class eggmock_mig_ffi;
  class eggmock_mig_ffi_callback;

  class eggmock_mig_ffi
  {
  public:
    ~eggmock_mig_ffi() { free( data ); }

  private:
    eggmock_mig_ffi() = default;
    void* data;
    uint64_t ( *add_symbol )( void* data, uint64_t name );
    uint64_t ( *add_true )( void* data );
    uint64_t ( *add_false )( void* data );
    uint64_t ( *add_not )( void* data, uint64_t id1 );
    uint64_t ( *add_maj )( void* data, uint64_t id1, uint64_t id2, uint64_t id3 );
    void ( *rewrite )( void* data, size_t roots_size, const uint64_t* roots, eggmock_mig_ffi_callback callback );
    void ( *free )( void* data );
  };

  struct eggmock_mig_ffi_callback
  {
    void* data;
    uint64_t ( *add_symbol )( void* data, uint64_t name );
    uint64_t ( *add_true )( void* data );
    uint64_t ( *add_false )( void* data );
    uint64_t ( *add_not )( void* data, uint64_t id1 );
    uint64_t ( *add_maj )( void* data, uint64_t id1, uint64_t id2, uint64_t id3 );
    void ( *mark_roots )( void* data, size_t roots_size, const uint64_t* roots );

    ~eggmock_mig_ffi_callback()
    {
      free( data );
    }
  };

#ifdef __cplusplus
}
#endif

class mig_ffi
{
public:
  explicit mig_ffi( const eggmock_mig_ffi& ffi ) : ffi_( ffi ) {}
  ~mig_ffi()
  {
    ffi_.free( ffi_.data );
  }

  mig_ffi( const mig_ffi& ffi ) = delete;
  mig_ffi( mig_ffi&& ) = delete;
  mig_ffi& operator=( const mig_ffi& ) = delete;
  mig_ffi& operator=( mig_ffi&& ) = delete;



private:
  eggmock_mig_ffi ffi_;
};

int main( int argc )
{
  mockturtle::mig_network ntk = mockturtle::mig_network();
  std::cout
      << "Hello, World!" << std::endl;
  std::cout
      << "Hello, World!" << std::endl;
  return 0;
}

struct What
{
};

void f()
{
  What w;
  w.f()
}
