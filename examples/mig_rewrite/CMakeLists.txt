corrosion_import_crate(MANIFEST_PATH rs/Cargo.toml)

add_executable(mig_rewrite main.cpp)

target_link_libraries(mig_rewrite PRIVATE mockturtle)
target_link_libraries(mig_rewrite PRIVATE mig_rewrite_rs)