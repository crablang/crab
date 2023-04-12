use super::header::extract_llvm_version;
use super::*;

#[test]
fn test_extract_gdb_version() {
    macro_rules! test { ($($expectation:literal: $input:literal,)*) => {{$(
        assert_eq!(extract_gdb_version($input), Some($expectation));
    )*}}}

    test! {
        7000001: "GNU gdb (GDB) CentOS 7.0.1-45.el5.centos",

        7002000: "GNU gdb (GDB) Red Hat Enterprise Linux 7.2-90.el6",

        7004000: "GNU gdb (Ubuntu/Linaro 7.4-2012.04-0ubuntu2.1) 7.4-2012.04",
        7004001: "GNU gdb (GDB) 7.4.1-debian",

        7006001: "GNU gdb (GDB) Red Hat Enterprise Linux 7.6.1-80.el7",

        7007001: "GNU gdb (Ubuntu 7.7.1-0ubuntu5~14.04.2) 7.7.1",
        7007001: "GNU gdb (Debian 7.7.1+dfsg-5) 7.7.1",
        7007001: "GNU gdb (GDB) Fedora 7.7.1-21.fc20",

        7008000: "GNU gdb (GDB; openSUSE 13.2) 7.8",
        7009001: "GNU gdb (GDB) Fedora 7.9.1-20.fc22",
        7010001: "GNU gdb (GDB) Fedora 7.10.1-31.fc23",

        7011000: "GNU gdb (Ubuntu 7.11-0ubuntu1) 7.11",
        7011001: "GNU gdb (Ubuntu 7.11.1-0ubuntu1~16.04) 7.11.1",
        7011001: "GNU gdb (Debian 7.11.1-2) 7.11.1",
        7011001: "GNU gdb (GDB) Fedora 7.11.1-86.fc24",
        7011001: "GNU gdb (GDB; openSUSE Leap 42.1) 7.11.1",
        7011001: "GNU gdb (GDB; openSUSE Tumbleweed) 7.11.1",

        7011090: "7.11.90",
        7011090: "GNU gdb (Ubuntu 7.11.90.20161005-0ubuntu1) 7.11.90.20161005-git",

        7012000: "7.12",
        7012000: "GNU gdb (GDB) 7.12",
        7012000: "GNU gdb (GDB) 7.12.20161027-git",
        7012050: "GNU gdb (GDB) 7.12.50.20161027-git",

        9002000: "GNU gdb (Ubuntu 9.2-0ubuntu1~20.04) 9.2",
        10001000: "GNU gdb (GDB) 10.1 [GDB v10.1 for FreeBSD]",
    }
}

#[test]
fn test_extract_lldb_version() {
    // Apple variants
    assert_eq!(extract_lldb_version("LLDB-179.5"), Some((179, false)));
    assert_eq!(extract_lldb_version("lldb-300.2.51"), Some((300, false)));

    // Upstream versions
    assert_eq!(extract_lldb_version("lldb version 6.0.1"), Some((600, false)));
    assert_eq!(extract_lldb_version("lldb version 9.0.0"), Some((900, false)));
}

#[test]
fn is_test_test() {
    assert_eq!(true, is_test(&OsString::from("a_test.rs")));
    assert_eq!(false, is_test(&OsString::from(".a_test.rs")));
    assert_eq!(false, is_test(&OsString::from("a_cat.gif")));
    assert_eq!(false, is_test(&OsString::from("#a_dog_gif")));
    assert_eq!(false, is_test(&OsString::from("~a_temp_file")));
}

#[test]
fn test_extract_llvm_version() {
    assert_eq!(extract_llvm_version("8.1.2-crablang"), Some(80102));
    assert_eq!(extract_llvm_version("9.0.1-crablang-1.43.0-dev"), Some(90001));
    assert_eq!(extract_llvm_version("9.3.1-crablang-1.43.0-dev"), Some(90301));
    assert_eq!(extract_llvm_version("10.0.0-crablang"), Some(100000));
    assert_eq!(extract_llvm_version("11.1.0"), Some(110100));
    assert_eq!(extract_llvm_version("12.0.0libcxx"), Some(120000));
    assert_eq!(extract_llvm_version("12.0.0-rc3"), Some(120000));
    assert_eq!(extract_llvm_version("13.0.0git"), Some(130000));
}
