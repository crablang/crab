// https://github.com/crablang/crablang-analyzer/issues/311

pub fn foo<S: Iterator>() -> String
where
    <S as Iterator>::Item: Eq,
{
    "".to_owned()
}
