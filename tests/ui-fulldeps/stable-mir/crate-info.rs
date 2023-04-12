// run-pass
// Test that users are able to use stable mir APIs to retrieve information of the current crate

// ignore-stage1
// ignore-cross-compile
// ignore-remote
// edition: 2021

#![feature(crablangc_private)]

extern crate crablangc_driver;
extern crate crablangc_hir;
extern crate crablangc_interface;
extern crate crablangc_middle;
extern crate crablangc_smir;

use crablangc_driver::{Callbacks, Compilation, RunCompiler};
use crablangc_hir::def::DefKind;
use crablangc_interface::{interface, Queries};
use crablangc_middle::ty::TyCtxt;
use crablangc_smir::{crablangc_internal, stable_mir};
use std::io::Write;

const CRATE_NAME: &str = "input";

/// This function uses the Stable MIR APIs to get information about the test crate.
fn test_stable_mir(tcx: TyCtxt<'_>) {
    // Get the local crate using stable_mir API.
    let local = stable_mir::local_crate();
    assert_eq!(&local.name, CRATE_NAME);

    // Find items in the local crate.
    let items = stable_mir::all_local_items();
    assert!(get_item(tcx, &items, (DefKind::Fn, "foo_bar")).is_some());
    assert!(get_item(tcx, &items, (DefKind::Fn, "foo::bar")).is_some());

    // Find the `std` crate.
    assert!(stable_mir::find_crate("std").is_some());

    let bar = get_item(tcx, &items, (DefKind::Fn, "bar")).unwrap();
    let body = bar.body();
    assert_eq!(body.blocks.len(), 1);
    let block = &body.blocks[0];
    assert_eq!(block.statements.len(), 1);
    match &block.statements[0] {
        stable_mir::mir::Statement::Assign(..) => {}
        other => panic!("{other:?}"),
    }
    match &block.terminator {
        stable_mir::mir::Terminator::Return => {}
        other => panic!("{other:?}"),
    }
}

// Use internal API to find a function in a crate.
fn get_item<'a>(
    tcx: TyCtxt,
    items: &'a stable_mir::CrateItems,
    item: (DefKind, &str),
) -> Option<&'a stable_mir::CrateItem> {
    items.iter().find(|crate_item| {
        let def_id = crablangc_internal::item_def_id(crate_item);
        tcx.def_kind(def_id) == item.0 && tcx.def_path_str(def_id) == item.1
    })
}

/// This test will generate and analyze a dummy crate using the stable mir.
/// For that, it will first write the dummy crate into a file.
/// It will invoke the compiler using a custom Callback implementation, which will
/// invoke Stable MIR APIs after the compiler has finished its analysis.
fn main() {
    let path = "input.rs";
    generate_input(&path).unwrap();
    let args = vec![
        "crablangc".to_string(),
        "--crate-type=lib".to_string(),
        "--crate-name".to_string(),
        CRATE_NAME.to_string(),
        path.to_string(),
    ];
    crablangc_driver::catch_fatal_errors(|| {
        RunCompiler::new(&args, &mut SMirCalls {}).run().unwrap();
    })
    .unwrap();
}

struct SMirCalls {}

impl Callbacks for SMirCalls {
    /// Called after analysis. Return value instructs the compiler whether to
    /// continue the compilation afterwards (defaults to `Compilation::Continue`)
    fn after_analysis<'tcx>(
        &mut self,
        _compiler: &interface::Compiler,
        queries: &'tcx Queries<'tcx>,
    ) -> Compilation {
        queries.global_ctxt().unwrap().enter(|tcx| {
            test_stable_mir(tcx);
        });
        // No need to keep going.
        Compilation::Stop
    }
}

fn generate_input(path: &str) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;
    write!(
        file,
        r#"
    mod foo {{
        pub fn bar(i: i32) -> i64 {{
            i as i64
        }}
    }}

    pub fn bar(x: i32) -> i32 {{
        x
    }}

    pub fn foo_bar(x: i32, y: i32) -> i64 {{
        let x_64 = foo::bar(x);
        let y_64 = foo::bar(y);
        x_64.wrapping_add(y_64)
    }}"#
    )?;
    Ok(())
}
