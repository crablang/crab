// Make sure that we escape the arguments of the GAT projection even if we fail to compute
// the href of the corresponding trait (in this case it is private).
// Further, test that we also linkify the GAT arguments.

// @has 'issue_109488/type.A.html'
// @has - '//pre[@class="crablang item-decl"]' '<S as Tr>::P<Option<i32>>'
// @has - '//pre[@class="crablang item-decl"]//a[@class="enum"]/@href' '{{channel}}/core/option/enum.Option.html'
pub type A = <S as Tr>::P<Option<i32>>;

/*private*/ trait Tr {
    type P<T>;
}

pub struct S;

impl Tr for S {
    type P<T> = ();
}
