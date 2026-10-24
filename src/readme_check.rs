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

/// Source text before the first `#[cfg(test)]`, so tests don't count as users.
fn non_test(src: &str) -> &str {
    src.split("#[cfg(test)]").next().unwrap_or(src)
}

fn rs_files(dir: &std::path::Path, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            rs_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(std::fs::read_to_string(&p).expect("read source file"));
        }
    }
}

fn has_word(hay: &str, word: &str) -> bool {
    hay.match_indices(word).any(|(i, _)| {
        let ok = |c: Option<char>| c.is_none_or(|c| !(c.is_alphanumeric() || c == '_'));
        ok(hay[..i].chars().next_back()) && ok(hay[i + word.len()..].chars().next())
    })
}

/// The README says every module is used by a binary or example. Check it:
/// walk from the non-test code of src/bin and examples through the modules
/// they name (`m::…` or an item lib.rs re-exports from `m`).
#[test]
fn every_library_module_is_reached_from_a_binary_or_example() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib = include_str!("lib.rs");
    let mods: Vec<&str> = lib
        .lines()
        .filter_map(|l| l.trim().strip_prefix("pub mod "))
        .filter_map(|l| l.strip_suffix(';'))
        .collect();
    // Names that point at module m from outside it.
    let names = |m: &str| -> Vec<String> {
        let mut v = vec![format!("{m}::")];
        for l in lib.lines() {
            let Some(rest) = l.trim().strip_prefix(&format!("pub use {m}::")) else {
                continue;
            };
            let rest = rest
                .trim_end_matches(';')
                .trim_matches(|c| c == '{' || c == '}');
            if rest == "*" {
                let src = std::fs::read_to_string(root.join(format!("src/{m}.rs"))).expect("read");
                for l in non_test(&src).lines() {
                    for kw in [
                        "pub struct ",
                        "pub enum ",
                        "pub fn ",
                        "pub trait ",
                        "pub const ",
                    ] {
                        if let Some(n) = l.strip_prefix(kw) {
                            let n: String = n
                                .chars()
                                .take_while(|c| c.is_alphanumeric() || *c == '_')
                                .collect();
                            v.push(n);
                        }
                    }
                }
            } else {
                for it in rest.split(',') {
                    if let Some(n) = it.split(" as ").last().map(str::trim) {
                        if !n.is_empty() {
                            v.push(n.to_string());
                        }
                    }
                }
            }
        }
        v
    };
    let refers = |src: &str, m: &str| {
        names(m).iter().any(|n| {
            if let Some(stem) = n.strip_suffix("::") {
                // `m::` as a path segment, or `m` inside a `use …::{…}` group.
                src.match_indices(n.as_str()).any(|(i, _)| {
                    src[..i]
                        .chars()
                        .next_back()
                        .is_none_or(|c| !(c.is_alphanumeric() || c == '_'))
                }) || src.contains(&format!("{{{stem},"))
                    || src.contains(&format!(" {stem},"))
                    || src.contains(&format!("{{self, {stem}"))
            } else {
                has_word(src, n)
            }
        })
    };
    let mut users = Vec::new();
    rs_files(&root.join("src/bin"), &mut users);
    rs_files(&root.join("examples"), &mut users);
    let roots: String = users
        .iter()
        .map(|s| non_test(s))
        .collect::<Vec<_>>()
        .join("\n");
    let mut reached: Vec<&str> = mods.iter().copied().filter(|m| refers(&roots, m)).collect();
    let mut i = 0;
    while i < reached.len() {
        let src = std::fs::read_to_string(root.join(format!("src/{}.rs", reached[i])))
            .expect("read module");
        for m in &mods {
            if !reached.contains(m) && refers(non_test(&src), m) {
                reached.push(m);
            }
        }
        i += 1;
    }
    let unused: Vec<&&str> = mods.iter().filter(|m| !reached.contains(m)).collect();
    assert!(
        unused.is_empty(),
        "lib modules no binary or example reaches: {unused:?}"
    );
}
