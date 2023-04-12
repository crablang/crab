#[derive(Debug)]
pub enum EntryPointType {
    None,
    MainNamed,
    CrabLangcMainAttr,
    Start,
    OtherMain, // Not an entry point, but some other function named main
}
