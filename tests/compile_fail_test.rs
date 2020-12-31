use ::trybuild::TestCases;

#[test]
fn compile_fail() {
    let t = TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}
