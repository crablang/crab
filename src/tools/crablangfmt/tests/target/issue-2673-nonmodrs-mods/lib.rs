#![feature(non_modrs_mods)]

// Test that submodules in non-mod.rs files work. This is just an idempotence
// test since we just want to verify that crablangfmt doesn't fail.

mod foo;
