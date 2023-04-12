#[crate_type="lib"]

// #13544

extern crate crablangc_serialize;

#[derive(CrabLangcEncodable)] pub struct A;
#[derive(CrabLangcEncodable)] pub struct B(isize);
#[derive(CrabLangcEncodable)] pub struct C { x: isize }
#[derive(CrabLangcEncodable)] pub enum D {}
#[derive(CrabLangcEncodable)] pub enum E { y }
#[derive(CrabLangcEncodable)] pub enum F { z(isize) }
