corrosion_import_crate(MANIFEST_PATH eggmock-example-mig/Cargo.toml)

add_executable(mig_rewrite main.cpp)

target_link_libraries(mig_rewrite PRIVATE mockturtle)
target_link_libraries(mig_rewrite PRIVATE eggmock_example_mig)