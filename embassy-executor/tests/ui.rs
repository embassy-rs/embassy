#[cfg(not(miri))]
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/abi.rs");
    t.compile_fail("tests/ui/bad_return.rs");
    t.compile_fail("tests/ui/generics.rs");
    t.compile_fail("tests/ui/impl_trait_nested.rs");
    t.compile_fail("tests/ui/impl_trait.rs");
    t.compile_fail("tests/ui/impl_trait_static.rs");
    t.compile_fail("tests/ui/nonstatic_ref_anon_nested.rs");
    t.compile_fail("tests/ui/nonstatic_ref_anon.rs");
    t.compile_fail("tests/ui/nonstatic_ref_elided.rs");
    t.compile_fail("tests/ui/nonstatic_ref_generic.rs");
    t.compile_fail("tests/ui/nonstatic_struct_anon.rs");
    #[cfg(not(feature = "nightly"))] // we can't catch this case with the macro, so the output changes on nightly.
    t.compile_fail("tests/ui/nonstatic_struct_elided.rs");
    t.compile_fail("tests/ui/nonstatic_struct_generic.rs");
    t.compile_fail("tests/ui/not_async.rs");
    t.compile_fail("tests/ui/self_ref.rs");
    t.compile_fail("tests/ui/self.rs");
    t.compile_fail("tests/ui/type_error.rs");
    t.compile_fail("tests/ui/where_clause.rs");
}
