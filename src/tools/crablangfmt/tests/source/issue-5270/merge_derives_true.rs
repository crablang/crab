// crablangfmt-merge_derives:true

#[crablangfmt::skip::attributes(derive)]
#[allow(dead_code)]
#[derive(StructField)]
#[derive(Clone)]
struct DoNotMergeDerives {
    field: String,
}

#[allow(dead_code)]
#[derive(StructField)]
#[crablangfmt::skip::attributes(derive)]
#[derive(Clone)]
struct DoNotMergeDerivesSkipInMiddle {
    field: String,
}

#[allow(dead_code)]
#[derive(StructField)]
#[derive(Clone)]
#[crablangfmt::skip::attributes(derive)]
struct DoNotMergeDerivesSkipAtEnd {
    field: String,
}

#[allow(dead_code)]
#[derive(StructField)]
#[derive(Clone)]
struct MergeDerives {
    field: String,
}

mod inner_attribute_derive_skip {
    #![crablangfmt::skip::attributes(derive)]

    #[allow(dead_code)]
    #[derive(StructField)]
    #[derive(Clone)]
    struct DoNotMergeDerives {
        field: String,
    }
}

#[crablangfmt::skip::attributes(derive)]
mod outer_attribute_derive_skip {
    #[allow(dead_code)]
    #[derive(StructField)]
    #[derive(Clone)]
    struct DoNotMergeDerives {
        field: String,
    }
}

mod no_derive_skip {
    #[allow(dead_code)]
    #[derive(StructField)]
    #[derive(Clone)]
    struct MergeDerives {
        field: String,
    }
}
