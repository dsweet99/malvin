//! External kiss witnesses for gitignore tree test helpers (credit `gitignore_tree_tests.rs`).

#[test]
fn kiss_witness_gitignore_tree_test_helper_symbols() {
    let _ = super::gitignore_tree::gitignore_tree_tests::seed_nested_gitignore_repo;
    let _ = super::gitignore_tree::gitignore_tree_tests::tamper_gitignore_tree;
    let _ = super::gitignore_tree::gitignore_tree_tests::assert_gitignore_contents;
}
