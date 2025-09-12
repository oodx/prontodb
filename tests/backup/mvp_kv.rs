use std::process::Command;
use std::time::Duration;
use std::thread::sleep;
use prontodb::xdg::test_utils::TestXdg;

fn bin() -> String { "./target/debug/prontodb".to_string() }

#[test]
fn mvp_set_get_del_basic() {
    // isolated env
    let test_xdg = TestXdg::new().expect("create xdg");
    let home = test_xdg.home_str();

    // set
    let status = Command::new(bin())
        .args(["set", "-p", "p1", "-n", "n1", "k", "v"]) 
        .env("HOME", home)
        .status().expect("run set");
    assert!(status.success());

    // get
    let out = Command::new(bin())
        .args(["get", "-p", "p1", "-n", "n1", "k"]) 
        .env("HOME", home)
        .output().expect("run get");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "v");

    // del
    let status = Command::new(bin())
        .args(["del", "-p", "p1", "-n", "n1", "k"]) 
        .env("HOME", home)
        .status().expect("run del");
    assert!(status.success());

    // get miss
    let out = Command::new(bin())
        .args(["get", "-p", "p1", "-n", "n1", "k"]) 
        .env("HOME", home)
        .output().expect("run get miss");
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}

#[test]
fn mvp_keys_scan_prefix() {
    let test_xdg = TestXdg::new().expect("create xdg");
    let home = test_xdg.home_str();

    // seed keys
    for (k, v) in [("a1","v1"),("a2","v2"),("b1","v3")] {
        let status = Command::new(bin())
            .args(["set", "-p", "p2", "-n", "n2", k, v])
            .env("HOME", home)
            .status().expect("run set");
        assert!(status.success());
    }

    // keys all
    let out = Command::new(bin())
        .args(["keys", "-p", "p2", "-n", "n2"]) 
        .env("HOME", home)
        .output().expect("run keys");
    assert_eq!(out.status.code(), Some(0));
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("a1"));
    assert!(s.contains("a2"));
    assert!(s.contains("b1"));

    // keys prefix
    let out = Command::new(bin())
        .args(["keys", "-p", "p2", "-n", "n2", "a"]) 
        .env("HOME", home)
        .output().expect("run keys prefix");
    assert_eq!(out.status.code(), Some(0));
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("a1"));
    assert!(s.contains("a2"));
    assert!(!s.contains("b1"));

    // scan
    let out = Command::new(bin())
        .args(["scan", "-p", "p2", "-n", "n2"]) 
        .env("HOME", home)
        .output().expect("run scan");
    assert_eq!(out.status.code(), Some(0));
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("a1=v1"));
    assert!(s.contains("a2=v2"));
    assert!(s.contains("b1=v3"));
}

#[test]
fn mvp_ttl_rules_and_expiry() {
    let test_xdg = TestXdg::new().expect("create xdg");
    let home = test_xdg.home_str();

    // create TTL namespace
    let status = Command::new(bin())
        .args(["create-cache", "p3.n3", "1"]) 
        .env("HOME", home)
        .status().expect("run create-cache");
    assert!(status.success());

    // set without explicit ttl (uses default)
    let status = Command::new(bin())
        .args(["set", "-p", "p3", "-n", "n3", "k", "v"]) 
        .env("HOME", home)
        .status().expect("run set");
    assert!(status.success());

    // wait > default
    sleep(Duration::from_millis(1200));

    // get should MISS
    let out = Command::new(bin())
        .args(["get", "-p", "p3", "-n", "n3", "k"]) 
        .env("HOME", home)
        .output().expect("run get");
    assert_eq!(out.status.code(), Some(2));

    // ttl flag in non-TTL ns should error
    let status = Command::new(bin())
        .args(["set", "-p", "p4", "-n", "std", "k", "v", "--ttl", "5"]) 
        .env("HOME", home)
        .status().expect("run set ttl std");
    assert_ne!(status.code(), Some(0));
    assert_ne!(status.code(), Some(2));
}

