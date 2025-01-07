# eggmock

## FFI

Stage 1: transfer to E-Graph:
* `eggmock_[ntk_type]_rewriter(name) -> Rewriter`

```c++
struct Rewriter {
    void* (*create)(),
    uint64_t add_symbol(void*),
    uint64_t add_node_[type](void*, uint64_t const[])
    void (*rewrite)(void*, RewriteCallback callbacks)
}
```

Stage 2: perform E-Rewriting and get back result:
* `eggmock_[ntk_type]_rewrite(Ntk, NetworkCallbacks)`
```c++
struct NetworkCallbacks {
    void* data,
    uint64_t add_symbol(void*),
    uint64_t add_node_[type](void*, uint64_t const[])y
}
```