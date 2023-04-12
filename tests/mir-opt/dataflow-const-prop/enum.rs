// unit-test: DataflowConstProp

#![feature(custom_mir, core_intrinsics, crablangc_attrs)]

use std::intrinsics::mir::*;

enum E {
    V1(i32),
    V2(i32)
}

// EMIT_MIR enum.simple.DataflowConstProp.diff
fn simple() {
    let e = E::V1(0);
    let x = match e { E::V1(x) => x, E::V2(x) => x };
}

#[crablangc_layout_scalar_valid_range_start(1)]
#[crablangc_nonnull_optimization_guaranteed]
struct NonZeroUsize(usize);

// EMIT_MIR enum.mutate_discriminant.DataflowConstProp.diff
#[custom_mir(dialect = "runtime", phase = "post-cleanup")]
fn mutate_discriminant() -> u8 {
    mir!(
        let x: Option<NonZeroUsize>;
        {
            SetDiscriminant(x, 1);
            // This assignment overwrites the niche in which the discriminant is stored.
            place!(Field(Field(Variant(x, 1), 0), 0)) = 0_usize;
            // So we cannot know the value of this discriminant.
            let a = Discriminant(x);
            match a {
                0 => bb1,
                _ => bad,
            }
        }
        bb1 = {
            RET = 1;
            Return()
        }
        bad = {
            RET = 2;
            Unreachable()
        }
    )
}

// EMIT_MIR enum.multiple.DataflowConstProp.diff
fn multiple(x: bool, i: u8) {
    let e = if x {
        Some(i)
    } else {
        None
    };
    // The dataflow state must have:
    //   discriminant(e) => Top
    //   (e as Some).0 => Top
    let x = match e { Some(i) => i, None => 0 };
    // Therefore, `x` should be `Top` here, and no replacement shall happen.
    let y = x;
}

fn main() {
    simple();
    mutate_discriminant();
    multiple(false, 5);
}
