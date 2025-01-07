add_executable(mig_rewrite main.cpp)
target_link_libraries(mig_rewrite PRIVATE eggmock)
target_link_libraries(mig_rewrite PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}/eggmock-example-mig/target/debug/libeggmock_example_mig.so")