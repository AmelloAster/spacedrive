use nom::{
	bytes::complete::*, character::complete::*, combinator::*, error::ParseError, multi::*,
	sequence::*, IResult,
};

use super::Attribute;

fn remove_ws<'a, O, E: ParseError<&'a str>, F>(
	wrapped: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
	F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
	delimited(multispace0, wrapped, multispace0)
}

fn parens(input: &str) -> IResult<&str, &str> {
	delimited(char('('), is_not(")"), char(')'))(input)
}

fn attribute_field(input: &str) -> IResult<&str, (&str, &str)> {
	remove_ws(separated_pair(
		remove_ws(is_not(":")),
		char(':'),
		remove_ws(is_not(",")),
	))(input)
}

fn attribute_fields(input: &str) -> IResult<&str, Vec<(&str, &str)>> {
	separated_list1(char(','), attribute_field)(input)
}

pub fn parse(input: &str) -> IResult<&str, Attribute> {
	let (input, _) = remove_ws(tag("@"))(input)?;
	let (input, name) = alpha1(input)?;
	let (input, values_str) = opt(remove_ws(parens))(input)?;

	let fields = match values_str {
		Some(values_str) => attribute_fields(values_str)?.1,
		None => vec![],
	};

	Ok((input, Attribute { name, fields }))
}
