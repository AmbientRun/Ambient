#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Inline {
    Text(String),
    Field(String),
}
impl Inline {
    fn text(text: &str) -> Self {
        Self::Text(text.to_string())
    }
    fn field(field: &str) -> Self {
        Self::Field(field.to_string())
    }
}
pub fn parse_inline_string(inline: &str) -> Vec<Vec<Inline>> {
    inline
        .split('\n')
        .map(|row| {
            let mut res = Vec::new();
            parse_inline_string_inner(row, &mut res);
            res
        })
        .collect()
}
fn parse_inline_string_inner(inline: &str, res: &mut Vec<Inline>) {
    if let Some((start, end)) = inline.split_once('{') {
        if !start.is_empty() {
            res.push(Inline::text(start));
        }
        let (field, end) = end.split_once('}').expect("Missing end bracket in inline string");
        res.push(Inline::field(field));
        parse_inline_string_inner(end, res);
    } else if !inline.is_empty() {
        res.push(Inline::text(inline));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_inline_string() {
        assert_eq!(parse_inline_string(""), vec![vec![]]);
        assert_eq!(parse_inline_string("Hi"), vec![vec![Inline::text("Hi")]]);
        assert_eq!(parse_inline_string("Hi {test}"), vec![vec![Inline::text("Hi "), Inline::field("test")]]);
        assert_eq!(parse_inline_string("{test} hi"), vec![vec![Inline::field("test"), Inline::text(" hi")]]);
        assert_eq!(parse_inline_string("{test}"), vec![vec![Inline::field("test")]]);
        assert_eq!(parse_inline_string("Hi {test} hello"), vec![vec![Inline::text("Hi "), Inline::field("test"), Inline::text(" hello")]]);
        assert_eq!(
            parse_inline_string("Hi {test} hello {oh}"),
            vec![vec![Inline::text("Hi "), Inline::field("test"), Inline::text(" hello "), Inline::field("oh")]]
        );

        assert_eq!(parse_inline_string("Hi\ntest"), vec![vec![Inline::text("Hi")], vec![Inline::text("test")]]);
    }
}
