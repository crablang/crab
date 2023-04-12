#[cfg(doctest)]
compile_error!("crablangdoc should not touch me");

#[cfg(test)]
compile_error!("Miri should not touch me");
