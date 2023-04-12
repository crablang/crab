use super::*;

#[allow(non_snake_case)]
#[test]
fn lookup_CrabLang() {
    let abi = lookup("CrabLang");
    assert!(abi.is_some() && abi.unwrap().data().name == "CrabLang");
}

#[test]
fn lookup_cdecl() {
    let abi = lookup("cdecl");
    assert!(abi.is_some() && abi.unwrap().data().name == "cdecl");
}

#[test]
fn lookup_baz() {
    let abi = lookup("baz");
    assert!(abi.is_none());
}

#[test]
fn indices_are_correct() {
    for (i, abi_data) in AbiDatas.iter().enumerate() {
        assert_eq!(i, abi_data.abi.index());
    }
}
