\
/*!
ProntoDB TDD Scaffold
- Reads TEST-SPEC.md and implements grouped tests per feature bucket.
- Use PRONTODB_BIN to point at the binary; defaults to ./target/debug/prontodb
- Each module owns its own isolated HOME to avoid cross-talk.
*/

use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

// ---------- Harness ----------
fn bin_path() -> PathBuf {
    env::var_os("PRONTODB_BIN").map(PathBuf::from).unwrap_or_else(|| "./target/debug/prontodb".into())
}
fn mkhome(tag: &str) -> PathBuf {
    let base = env::temp_dir().join(format!("prontodb_tdd_{}_{}_{}", tag, std::process::id(), nano_ts()));
    fs::create_dir_all(&base).expect("mkdir temp home");
    base
}
fn nano_ts() -> u128 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
}
fn run(home: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(bin_path()).args(args).env("HOME", home).output().expect("spawn");
    (out.status.code().unwrap_or(-1), String::from_utf8_lossy(&out.stdout).into(), String::from_utf8_lossy(&out.stderr).into())
}
fn run_stream(home: &Path, stream: &str, extra: &[&str]) -> (i32, String, String) {
    let mut cmd = Command::new(bin_path());
    cmd.args(extra).arg("stream").env("HOME", home);
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn().expect("spawn stream");
    child.stdin.as_mut().unwrap().write_all(stream.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    (out.status.code().unwrap_or(-1), String::from_utf8_lossy(&out.stdout).into(), String::from_utf8_lossy(&out.stderr).into())
}
fn write_conf(home: &Path, body: &str) {
    let etc = home.join(".local/etc/odx/prontodb");
    fs::create_dir_all(&etc).unwrap();
    fs::write(etc.join("pronto.conf"), body).unwrap();
}

// ---------- Assertions ----------
fn assert_ok(code: i32, ctx: &str, stderr: &str) { assert!(code == 0, "{} failed: code={} stderr={}", ctx, code, stderr); }
fn assert_miss(code: i32, ctx: &str, stdout: &str, stderr: &str) { assert!(code == 2 && stdout.is_empty(), "{}: expected MISS (2); got code={}, stdout={}, stderr={}", ctx, code, stdout, stderr); }
fn assert_err(code: i32, ctx: &str) { assert!(code != 0 && code != 2, "{}: expected ERROR (!=0,!=2); got {}", ctx, code); }
fn contains(hay: &str, needle: &str, ctx: &str) { assert!(hay.contains(needle), "{}: missing '{}'\n{}", ctx, needle, hay); }

// =====================================
// 0) Lifecycle & Setup
// =====================================
mod lifecycle {
    use super::*;
    #[test]
    fn install_seeds_system_and_admin_v0_1() {
        let home = mkhome("install");
        let (c,_o,e) = run(&home, &["install"]);
        assert_ok(c, "install", &e);
        assert!(home.join(".local/etc/odx/prontodb").exists());
        assert!(home.join(".local/data/odx/prontodb").exists());
    }

    #[test]
    fn uninstall_and_purge_v0_1() {
        let home = mkhome("uninstall");
        assert_ok(run(&home, &["install"]).0, "install", "");
        let (c1,_o1,_e1) = run(&home, &["uninstall"]);
        assert_ok(c1, "uninstall", "");
        let (c2,_o2,_e2) = run(&home, &["install"]);
        assert_ok(c2, "reinstall", "");
        let (c3,_o3,_e3) = run(&home, &["uninstall","--purge"]);
        assert_ok(c3, "purge", "");
    }

    #[test]
    fn backup_snapshot_v0_1() {
        let home = mkhome("backup");
        assert_ok(run(&home, &["install"]).0, "install", "");
        let out = home.join("snap.tar.gz").to_string_lossy().to_string();
        let (c,_o,e) = run(&home, &["backup","--out", &out]);
        assert_ok(c, "backup", &e);
        assert!(Path::new(&out).exists(), "snapshot missing");
    }
}

// =====================================
// 1) Addressing, Keys, Delimiters
// =====================================
mod addressing {
    use super::*;
    #[test]
    fn canonical_full_path_v0_1() {
        let home = mkhome("addr_full");
        assert_ok(run(&home, &["install"]).0, "install", "");
        assert_ok(run(&home, &["set","kb.recipes.pasta__it","{\"sauce\":\"red\"}","--json"]).0, "set", "");
        let (c,o,e) = run(&home, &["get","kb.recipes.pasta__it","--json"]);
        assert_ok(c, "get", &e);
        contains(&o, "sauce", "json present");
    }
    #[test]
    fn flags_p_n_v0_1() {
        let home = mkhome("addr_flags");
        assert_ok(run(&home, &["install"]).0, "install", "");
        assert_ok(run(&home, &["-p","kb","-n","recipes","set","noodles","{\"w\":10}","--json"]).0, "set", "");
        let (c,o,e) = run(&home, &["-p","kb","-n","recipes","get","noodles","--json"]);
        assert_ok(c, "get", &e);
        contains(&o, "\"w\": 10", "json numeric");
    }
    #[test]
    fn ns_delim_override_v0_1() {
        let home = mkhome("addr_delim");
        assert_ok(run(&home, &["install"]).0, "install", "");
        assert_ok(run(&home, &["--ns-delim","|","set","kb|recipes|g","v"]).0, "set", "");
        let (c,o,e) = run(&home, &["--ns-delim","|","get","kb|recipes|g"]);
        assert_ok(c, "get", &e);
        contains(&o, "v", "value");
    }
    #[test]
    fn key_validation_no_delim_v0_1() {
        let home = mkhome("addr_keyval");
        assert_ok(run(&home, &["install"]).0, "install", "");
        let (c,_o,_e) = run(&home, &["-p","kb","-n","recipes","set","bad.key","x"]);
        assert_err(c, "delimiter not allowed in key");
    }
}

// =====================================
// 2) Core KV
// =====================================
mod kv {
    use super::*;
    #[test]
    fn set_get_string_v0_1() {
        let home = mkhome("kv_str");
        assert_ok(run(&home, &["install"]).0, "install", "");
        assert_ok(run(&home, &["set","kb.recipes.k","v"]).0, "set", "");
        let (c,o,e) = run(&home, &["get","kb.recipes.k"]);
        assert_ok(c, "get", &e);
        contains(&o, "v", "value");
    }
    #[test]
    fn delete_then_miss_v0_1() {
        let home = mkhome("kv_del");
        assert_ok(run(&home, &["install"]).0, "install", "");
        assert_ok(run(&home, &["set","kb.recipes.k","v"]).0, "set", "");
        assert_ok(run(&home, &["del","kb.recipes.k"]).0, "del", "");
        let (c,o,e) = run(&home, &["get","kb.recipes.k"]);
        assert_miss(c, "get after del", &o, &e);
    }
    #[test]
    fn scan_and_keys_v0_1() {
        let home = mkhome("kv_list");
        assert_ok(run(&home, &["install"]).0, "install", "");
        assert_ok(run(&home, &["set","kb.recipes.a","1"]).0, "set a", "");
        assert_ok(run(&home, &["set","kb.recipes.b","2"]).0, "set b", "");
        let (c1,o1,e1) = run(&home, &["-p","kb","-n","recipes","keys"]);
        assert_ok(c1, "keys", &e1);
        contains(&o1, "a", "keys contain a");
        contains(&o1, "b", "keys contain b");
        let (c2,o2,e2) = run(&home, &["-p","kb","-n","recipes","scan","--json"]);
        assert_ok(c2, "scan", &e2);
        contains(&o2, "\"k\"", "json k present");
        contains(&o2, "\"v\"", "json v present");
    }
}

// =====================================
// 3) TTL Namespaces
// =====================================
mod ttl {
    use super::*;
    #[test]
    fn create_ttl_and_expire_v0_1() {
        let home = mkhome("ttl_expire");
        assert_ok(run(&home, &["install"]).0, "install", "");
        assert_ok(run(&home, &["admin","create-cache","kb.recipes","timeout=2"]).0, "create-cache", "");
        assert_ok(run(&home, &["set","kb.recipes.temp","X"]).0, "set", ""); // default ttl=2s
        sleep(Duration::from_secs(3));
        let (c,o,e) = run(&home, &["get","kb.recipes.temp"]);
        assert_miss(c, "expired", &o, &e);
    }

    #[test]
    #[ignore] // v0.2 enforcement path
    fn ttl_flag_only_in_ttl_ns_v0_2() {
        let home = mkhome("ttl_flag");
        assert_ok(run(&home, &["install"]).0, "install", "");
        let (c,_o,_e) = run(&home, &["set","kb.standard.k","x","--ttl","5"]);
        assert_err(c, "ttl not allowed in std ns");
    }
}

// =====================================
// 4) Streams
// =====================================
mod streams {
    use super::*;
    #[test]
    fn auth_required_by_default_v0_1() {
        let home = mkhome("stream_auth");
        assert_ok(run(&home, &["install"]).0, "install", "");
        let (c,_o,_e) = run_stream(&home, "meta:path=kb.recipes; a=b;", &[]);
        assert_err(c, "auth required");
    }
    #[test]
    fn auth_order_enforced_v0_1() {
        let home = mkhome("stream_order");
        assert_ok(run(&home, &["install"]).0, "install", "");
        let bad = "meta:sec:user=admin; meta:sec:pass=pronto!; meta:path=kb.recipes; x=y;";
        let (c1,_o1,_e1) = run_stream(&home, bad, &[]);
        assert_err(c1, "reversed order error");
        let good = "meta:sec:pass=pronto!; meta:sec:user=admin; meta:path=kb.recipes; x=y;";
        let (c2,_o2,e2) = run_stream(&home, good, &[]);
        assert_ok(c2, "stream ok", &e2);
        let (_cg, og, _eg) = run(&home, &["get","kb.recipes.x"]);
        contains(&og, "y", "stream write");
    }
    #[test]
    fn disable_security_v0_1() {
        let home = mkhome("stream_nosec");
        assert_ok(run(&home, &["install"]).0, "install", "");
        write_conf(&home, "ns_delim=\".\"\nsecurity.required=false\n");
        let s = "meta:path=kb.recipes; x=y;";
        let (c,_o,e) = run_stream(&home, s, &[]);
        assert_ok(c, "stream", &e);
        let (_cg, og, _eg) = run(&home, &["get","kb.recipes.x"]);
        contains(&og, "y", "wrote");
    }

    #[test]
    #[ignore] // v0.2
    fn meta_ns_alias_v0_2() {
        let home = mkhome("stream_ns_alias");
        assert_ok(run(&home, &["install"]).0, "install", "");
        let s = "meta:sec:pass=pronto!; meta:sec:user=admin; meta:ns=kb.recipes; k=v;";
        let (c,_o,e) = run_stream(&home, s, &[]);
        assert_ok(c, "stream", &e);
    }
}

// =====================================
// 5) Discovery & Admin
// =====================================
mod discovery {
    use super::*;
    #[test]
    fn list_projects_and_namespaces_v0_1() {
        let home = mkhome("discover");
        assert_ok(run(&home, &["install"]).0, "install", "");
        assert_ok(run(&home, &["set","kb.recipes.alpha","1"]).0, "seed", "");
        let (_cp,p,_ep) = run(&home, &["projects"]);
        contains(&p, "kb", "projects include kb");
        let (_cn,n,_en) = run(&home, &["namespaces","-p","kb"]);
        contains(&n, "recipes", "namespaces include recipes");
        let (_ca,a,_ea) = run(&home, &["nss"]);
        contains(&a, "kb.recipes", "nss includes");
    }
}

// =====================================
// 6) Export/Import/Backup
// =====================================
mod ioops {
    use super::*;
    #[test]
    #[ignore] // v0.2
    fn tsv_roundtrip_v0_2() {
        let home = mkhome("tsv");
        assert_ok(run(&home, &["install"]).0, "install", "");
        assert_ok(run(&home, &["set","kb.recipes.bin","\u{0001}\u{0002}"]).0, "set bin", "");
        let (_ce, exp, _ee) = run(&home, &["export","kb.recipes"]);
        assert!(exp.contains("bin"));
        assert_ok(run(&home, &["uninstall","--purge"]).0, "purge", "");
        assert_ok(run(&home, &["install"]).0, "reinstall", "");
        // import via stdin
        let mut cmd = Command::new(bin_path());
        cmd.arg("import").arg("kb.recipes").env("HOME", &home);
        cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = cmd.spawn().unwrap();
        child.stdin.as_mut().unwrap().write_all(exp.as_bytes()).unwrap();
        let out = child.wait_with_output().unwrap();
        assert!(out.status.success(), "import failed: {}", String::from_utf8_lossy(&out.stderr));
        let (_cg, og, _eg) = run(&home, &["get","kb.recipes.bin"]);
        assert!(!og.is_empty(), "imported value should exist");
    }

    #[test]
    fn backup_snapshot_v0_1() {
        let home = mkhome("backup2");
        assert_ok(run(&home, &["install"]).0, "install", "");
        let out = home.join("snap2.tar.gz").to_string_lossy().to_string();
        let (c,_o,e) = run(&home, &["backup","--out", &out]);
        assert_ok(c, "backup", &e);
        assert!(Path::new(&out).exists());
    }
}

// =====================================
// 7) Exit Codes
// =====================================
mod exitcodes {
    use super::*;
    #[test]
    fn miss_is_2_v0_1() {
        let home = mkhome("exit_miss");
        assert_ok(run(&home, &["install"]).0, "install", "");
        let (c,o,e) = run(&home, &["get","kb.recipes.nope"]);
        assert_miss(c, "miss", &o, &e);
    }
}
