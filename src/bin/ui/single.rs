//! One window per user: a second plain launch focuses the running one.
//!
//! The first instance listens on a loopback port derived from the user
//! name. A later launch connects, says hello with the same user name, and
//! exits once the first instance answers; the first instance raises its
//! window. Anything unexpected (port taken by another program, another
//! user's instance, no answer) just lets the new instance start, so this
//! can never stop the app from opening.
//!
//! Skipped with `SERAPHINA_MULTI_INSTANCE=1`, when a file or `--run` script
//! is passed (so "Open with" still opens the file), and for the renderer
//! fallback relaunch (which waits for the old process to let go).

use std::io::{BufRead, BufReader, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::time::Duration;

use eframe::egui;

const HELLO: &str = "SERAPHINA-STUDIO-FOCUS";
const OK: &str = "SERAPHINA-STUDIO-OK";

/// Set by the listener; the UI raises the window and clears it.
pub static FOCUS_REQUESTED: AtomicBool = AtomicBool::new(false);
static CTX: OnceLock<egui::Context> = OnceLock::new();

/// Lets the listener wake the UI.
pub fn set_context(ctx: &egui::Context) {
    let _ = CTX.set(ctx.clone());
}

fn user() -> String {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_default()
}

/// A stable port per user name in 41000..=48999 (FNV-1a).
pub fn port_for(user: &str) -> u16 {
    let mut h: u32 = 0x811c_9dc5;
    for b in user.to_lowercase().bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    41_000 + (h % 8_000) as u16
}

fn addr(port: u16) -> SocketAddr {
    SocketAddr::from((Ipv4Addr::LOCALHOST, port))
}

#[derive(Debug, PartialEq, Eq)]
pub enum Start {
    /// We hold the port and will raise the window on request.
    First,
    /// Another instance of ours answered and was asked to come forward.
    Focused,
    /// Couldn't tell (port used by something else, no answer): start anyway.
    Unknown(String),
}

/// Claim the port, or ask the running instance to come forward.
pub fn claim(port: u16, user: &str, wait_for_release: bool) -> Start {
    let tries = if wait_for_release { 50 } else { 1 };
    for i in 0..tries {
        if let Ok(l) = TcpListener::bind(addr(port)) {
            serve(l, user.to_string());
            return Start::First;
        }
        if wait_for_release && i + 1 < tries {
            std::thread::sleep(Duration::from_millis(100));
        }
    }
    if wait_for_release {
        return Start::Unknown("the previous window is still closing".into());
    }
    match ask(port, user) {
        Ok(true) => Start::Focused,
        Ok(false) => Start::Unknown(format!("port {} answered but isn't ours", port)),
        Err(e) => Start::Unknown(format!("no answer on port {} ({})", port, e)),
    }
}

fn ask(port: u16, user: &str) -> std::io::Result<bool> {
    let mut s = TcpStream::connect_timeout(&addr(port), Duration::from_millis(800))?;
    s.set_read_timeout(Some(Duration::from_millis(1500)))?;
    s.set_write_timeout(Some(Duration::from_millis(800)))?;
    writeln!(s, "{} {}", HELLO, user)?;
    let mut line = String::new();
    BufReader::new(s).read_line(&mut line)?;
    Ok(line.trim() == OK)
}

fn serve(l: TcpListener, user: String) {
    std::thread::Builder::new()
        .name("single-instance".into())
        .spawn(move || {
            for s in l.incoming().flatten() {
                let _ = s.set_read_timeout(Some(Duration::from_millis(1000)));
                let mut line = String::new();
                if BufReader::new(&s).read_line(&mut line).is_err() {
                    continue;
                }
                let want = format!("{} {}", HELLO, user);
                if line.trim_end() != want {
                    continue; // not ours, or another user's launch
                }
                // Flag first, then answer: the caller may act on the reply at once.
                FOCUS_REQUESTED.store(true, Ordering::SeqCst);
                if let Some(c) = CTX.get() {
                    c.request_repaint();
                }
                let mut w = &s;
                let _ = writeln!(w, "{}", OK);
            }
        })
        .ok();
}

/// Called from `main`: true when this process should exit because the
/// running instance was brought forward.
pub fn defer_to_running(has_file_or_script: bool) -> bool {
    if has_file_or_script || std::env::var("SERAPHINA_MULTI_INSTANCE").is_ok() {
        return false;
    }
    let relaunch = std::env::var("SERAPHINA_RENDERER_FALLBACK").is_ok();
    let u = user();
    match claim(port_for(&u), &u, relaunch) {
        Start::First => false,
        Start::Focused => true,
        Start::Unknown(why) => {
            eprintln!("single-instance check skipped: {}", why);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn free_port() -> u16 {
        TcpListener::bind(addr(0))
            .unwrap()
            .local_addr()
            .unwrap()
            .port()
    }

    #[test]
    fn port_is_stable_and_in_range() {
        assert_eq!(port_for("Ana"), port_for("ana"));
        for u in ["", "ana", "bob", "a-very-long-user-name"] {
            assert!((41_000..49_000).contains(&port_for(u)));
        }
    }

    #[test]
    fn second_launch_focuses_first_and_other_users_are_ignored() {
        let port = free_port();
        assert_eq!(claim(port, "ana", false), Start::First);
        FOCUS_REQUESTED.store(false, Ordering::SeqCst);
        assert_eq!(claim(port, "ana", false), Start::Focused);
        assert!(FOCUS_REQUESTED.swap(false, Ordering::SeqCst));
        assert!(matches!(claim(port, "bob", false), Start::Unknown(_)));
        assert!(!FOCUS_REQUESTED.load(Ordering::SeqCst));
    }

    #[test]
    fn a_foreign_listener_never_blocks_startup() {
        let l = TcpListener::bind(addr(0)).unwrap();
        let port = l.local_addr().unwrap().port();
        std::thread::spawn(move || {
            for s in l.incoming().flatten() {
                let mut w = &s;
                let _ = writeln!(w, "HTTP/1.0 400 nope");
            }
        });
        assert!(matches!(claim(port, "ana", false), Start::Unknown(_)));
    }
}
