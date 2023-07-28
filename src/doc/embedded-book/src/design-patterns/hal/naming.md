# Naming


<a id="c-crate-name"></a>
## The crate is named appropriately (C-CRATE-NAME)

HAL crates should be named after the chip or family of chips they aim to
support. Their name should end with `-hal` to distinguish them from register
access crates. The name should not contain underscores (use dashes instead).
