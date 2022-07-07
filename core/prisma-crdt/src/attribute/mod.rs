mod parser;

pub struct Attribute<'a> {
	pub name: &'a str,
	pub fields: Vec<(&'a str, &'a str)>,
}

impl<'a> Attribute<'a> {
	pub fn parse(input: &'a impl AsRef<str>) -> Result<Self, ()> {
		parser::parse(input.as_ref())
			.map(|(_, a)| a)
			.map_err(|_| ())
	}

	pub fn field(&self, name: &str) -> Option<&str> {
		self.fields
			.iter()
			.find(|(n, _)| *n == name)
			.map(|(_, v)| *v)
	}
}
