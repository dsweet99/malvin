//! External kiss witnesses for vision tree test helpers (credit `vision_tree_tests.rs`).

#[test]
fn kiss_witness_vision_tree_test_helper_symbols() {
    let _ = super::vision_tree::vision_tree_tests::seed_nested_vision_repo;
    let _ = super::vision_tree::vision_tree_tests::tamper_vision_tree;
    let _ = super::vision_tree::vision_tree_tests::assert_vision_contents;
    let _ = super::tree_test_support::init_git_repo;
}
