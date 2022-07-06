use nom::{
	branch::*, bytes::complete::*, character::complete::*, combinator::*, error::ParseError,
	multi::*, sequence::*, IResult,
};

pub enum AttributeValue {
	String(String),
	Int(i32),
}

pub struct Attribute {
	name: String,
	fields: Vec<(String, AttributeValue)>,
}

fn wrap_ws<'a, O, E: ParseError<&'a str>, F>(
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

fn attribute_field(input: &str) -> IResult<&str, (String, AttributeValue)> {
	let (input, (field, value)) = wrap_ws(separated_pair(
		map(wrap_ws(is_not(":")), |v: &str| v.to_string()),
		char(':'),
		wrap_ws(is_not(",")),
	))(input)?;

	let (_, value) = alt((
		map(i32, AttributeValue::Int),
		map(alpha1, |s: &str| AttributeValue::String(s.to_string())),
	))(value)?;

	Ok((input, (field, value)))
}

fn attribute_fields(input: &str) -> IResult<&str, Vec<(String, AttributeValue)>> {
	separated_list1(char(','), attribute_field)(input)
}

pub fn attribute(input: &str) -> IResult<&str, Attribute> {
	let (input, _) = tag("@")(input)?;
	let (input, name) = alpha1(input)?;
	let (input, values_str) = opt(parens)(input)?;

	let fields = match values_str {
		Some(values_str) => attribute_fields(values_str)?.1,
		None => vec![],
	};

	Ok((
		input,
		Attribute {
			name: name.to_string(),
			fields,
		},
	))
}
