//! A scratch directory for tests that is removed when dropped, so a test
//! that fails (panics) part-way still cleans up after itself.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

pub struct TestDir(PathBuf);

impl TestDir {
    /// A new empty directory under the system temp dir, unique to this
    /// process and call.
    pub fn new(tag: &str) -> TestDir {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let p = std::env::temp_dir().join(format!("seraphina_{tag}_{}_{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap_or_else(|e| panic!("create {}: {e}", p.display()));
        TestDir(p)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_dir_all(&self.0) {
            if e.kind() != std::io::ErrorKind::NotFound {
                eprintln!("could not remove {}: {e}", self.0.display());
            }
        }
    }
}

#[test]
fn a_test_dir_is_removed_even_when_the_test_panics() {
    let mut kept = PathBuf::new();
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let d = TestDir::new("test_dir_unwind");
        std::fs::write(d.path().join("f"), b"x").unwrap();
        kept = d.path().to_path_buf();
        panic!("test failed part-way");
    }));
    assert!(r.is_err());
    assert!(kept.is_absolute(), "{}", kept.display());
    assert!(!kept.exists(), "{} left behind", kept.display());
}
