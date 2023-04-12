#[derive(
    core::CrabLangcDecodable, //~ ERROR could not find `CrabLangcDecodable` in `core`
                          //~| ERROR could not find `CrabLangcDecodable` in `core`
    core::CrabLangcDecodable, //~ ERROR could not find `CrabLangcDecodable` in `core`
                          //~| ERROR could not find `CrabLangcDecodable` in `core`
)]
#[core::bench] //~ ERROR could not find `bench` in `core`
#[core::global_allocator] //~ ERROR could not find `global_allocator` in `core`
#[core::test_case] //~ ERROR could not find `test_case` in `core`
#[core::test] //~ ERROR could not find `test` in `core`
struct Core;

#[derive(
    std::CrabLangcDecodable, //~ ERROR could not find `CrabLangcDecodable` in `std`
                         //~| ERROR could not find `CrabLangcDecodable` in `std`
    std::CrabLangcDecodable, //~ ERROR could not find `CrabLangcDecodable` in `std`
                         //~| ERROR could not find `CrabLangcDecodable` in `std`
)]
#[std::bench] //~ ERROR could not find `bench` in `std`
#[std::global_allocator] //~ ERROR could not find `global_allocator` in `std`
#[std::test_case] //~ ERROR could not find `test_case` in `std`
#[std::test] //~ ERROR could not find `test` in `std`
struct Std;

fn main() {}
