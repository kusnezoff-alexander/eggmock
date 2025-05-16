# eggmock

*eggmock* provides facilities to
- transfer and receive logic networks from [**mock**turtle](https://github.com/lsils/mockturtle) to and from Rust code with an automatically generated FFI and
- rewrite them using the [**egg**](https://github.com/egraphs-good/egg) library.

Currently, *eggmock* supports AIGs, MIGs, XMGs and XAGs.

## Prerequisites

To use *eggmock*, you need
- a modern C++ compiler and CMake
- a working Rust installation

If you want to test out if everything works as intended, you can run an example:

```shell
mkdir build
cd build
cmake ..
make mig_rewrite
./examples/mig_rewrite/mig_rewrite
```
- make sure to run `git submodule update --init --recursive` beforehand !

This will create two files:
- `in.dot` contains the logic network that was passed from mockturtle to egg
- `out.dot` contains the rewritten logic network

## Notes

- make sure to run a release build when measuring performance
- *eggmock* currently does not guarantee to be memory leak free in program runs that involve panics or exceptions
