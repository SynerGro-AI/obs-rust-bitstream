//! README.md lists exactly the modules declared in src/lib.rs.

#[test]
fn readme_lists_every_library_module() {
    let lib: Vec<&str> = include_str!("lib.rs")
        .lines()
        .filter_map(|l| l.trim().strip_prefix("pub mod "))
        .filter_map(|l| l.strip_suffix(';'))
        .collect();
    let readme = include_str!("../README.md");
    let block = readme
        .split("<!-- modules:begin -->")
        .nth(1)
        .and_then(|r| r.split("<!-- modules:end -->").next())
        .expect("README.md has a modules:begin/end block");
    let listed: Vec<&str> = block
        .lines()
        .filter_map(|l| l.strip_prefix("- `"))
        .filter_map(|l| l.split('`').next())
        .collect();
    let mut a = lib.clone();
    let mut b = listed.clone();
    a.sort_unstable();
    b.sort_unstable();
    assert_eq!(a, b, "README.md module list differs from src/lib.rs");
}
