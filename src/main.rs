use nom::{
    branch::alt,
    bytes::complete::{tag, take_till},
    character::complete::{digit1, multispace0},
    combinator::{eof, map},
    error::ParseError,
    multi::separated_list0,
    sequence::{delimited, tuple},
    IResult,
};
use std::fs;

#[derive(PartialEq, Eq, Debug)]
enum Json {
    Null,
    Bool(bool),
    Number(u64),
    String(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

fn json_null(input: &str) -> IResult<&str, Json> {
    map(tag("null"), |_| Json::Null)(input)
}

fn json_bool(input: &str) -> IResult<&str, Json> {
    let json_true = map(tag("true"), |_| Json::Bool(true));
    let json_false = map(tag("false"), |_| Json::Bool(false));
    alt((json_true, json_false))(input)
}

fn json_number(input: &str) -> IResult<&str, Json> {
    map(digit1, |s: &str| Json::Number(s.parse().unwrap()))(input)
}

fn string_literal(input: &str) -> IResult<&str, String> {
    let string = map(take_till(|c| c == '"'), |s: &str| s.into());
    delimited(tag("\""), string, tag("\""))(input)
}

fn json_string(input: &str) -> IResult<&str, Json> {
    map(string_literal, Json::String)(input)
}

fn json_array(input: &str) -> IResult<&str, Json> {
    map(
        delimited(
            ws(tag("[")),
            separated_list0(ws(tag(",")), json_value),
            ws(tag("]")),
        ),
        Json::Array,
    )(input)
}

fn json_object(input: &str) -> IResult<&str, Json> {
    map(
        delimited(
            ws(tag("{")),
            separated_list0(
                ws(tag(",")),
                tuple((string_literal, ws(tag(":")), json_value)),
            ),
            ws(tag("}")),
        ),
        |v| Json::Object(v.into_iter().map(|(k, _, v)| (k, v)).collect()),
    )(input)
}

fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

fn json_value(input: &str) -> IResult<&str, Json> {
    map(
        alt((
            json_null,
            json_bool,
            json_number,
            json_string,
            json_array,
            json_object,
        )),
        |json| json,
    )(input)
}

fn json(input: &str) -> IResult<&str, Json> {
    map(tuple((json_value, eof)), |(json, _)| json)(input)
}

fn read_json_file(filename: &str) -> Json {
    let s = fs::read_to_string(filename).unwrap();
    match json(&s) {
        Ok((_, json)) => json,
        Err(err) => panic!("{}", err.to_string()),
    }
}

fn main() {
    let json = read_json_file("test.json");

    println!("{:?}", json);
}

#[cfg(test)]
mod tests {
    use crate::{json_array, json_bool, json_null, json_number, json_object, json_string, Json};

    #[test]
    fn test_json_null() {
        assert_eq!(json_null("null").unwrap().1, Json::Null);
    }

    #[test]
    fn test_json_bool() {
        assert_eq!(json_bool("true").unwrap().1, Json::Bool(true));
        assert_eq!(json_bool("false").unwrap().1, Json::Bool(false));
    }

    #[test]
    fn test_json_number() {
        assert_eq!(json_number("1").unwrap().1, Json::Number(1));
        assert_eq!(json_number("123").unwrap().1, Json::Number(123));
    }

    #[test]
    fn test_json_string() {
        assert_eq!(json_string("\"\"").unwrap().1, Json::String("".into()));
        assert_eq!(json_string("\"a\"").unwrap().1, Json::String("a".into()));
    }

    #[test]
    fn test_json_array() {
        assert_eq!(json_array("[]").unwrap().1, Json::Array(vec![]));
        assert_eq!(
            json_array("[null]").unwrap().1,
            Json::Array(vec![Json::Null])
        );
        assert_eq!(
            json_array("[null, true, 1, \"a\"]").unwrap().1,
            Json::Array(vec![
                Json::Null,
                Json::Bool(true),
                Json::Number(1),
                Json::String("a".into())
            ])
        );
    }

    #[test]
    fn test_json_object() {
        assert_eq!(json_object("{}").unwrap().1, Json::Object(vec![]));
        assert_eq!(
            json_object("{\"key\": \"value\"}").unwrap().1,
            Json::Object(vec![("key".into(), Json::String("value".into()))])
        );
        assert_eq!(
            json_object("{\"outer\": {\"inner\": \"value\"}}")
                .unwrap()
                .1,
            Json::Object(vec![(
                "outer".into(),
                Json::Object(vec![("inner".into(), Json::String("value".into()))])
            )])
        );
    }
}
