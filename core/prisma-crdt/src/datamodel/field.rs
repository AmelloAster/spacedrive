use std::ops::Deref;

use crate::prelude::*;

#[derive(Debug)]
pub struct Field<'a> {
	pub prisma: &'a dml::Field,
	pub name_snake: Ident,
	pub name_pascal: Ident,
	pub typ: FieldType<'a>,
}

impl<'a> Field<'a> {
	pub fn resolve_relations(&self, datamodel: &Datamodel<'a>) {
		match &self.typ {
			FieldType::Scalar {
				relations: relations_referenced_by,
			} => {
				let relation_fields = datamodel
					.models
					.iter()
					.flat_map(|model| {
						model.fields.iter().filter_map(|f| {
							f.as_relation_field().and_then(|rf| {
								rf.relation_info
									.fields
									.iter()
									.any(|rf| rf == self.prisma.name())
									.then_some(f.clone())
							})
						})
					})
					.collect::<Vec<_>>();

				*relations_referenced_by.borrow_mut() =
					(!relation_fields.is_empty()).then_some(relation_fields);
			}
			_ => {}
		}
	}

	pub fn new(field: &'a dml::Field) -> Rc<Self> {
		Rc::new(Self {
			prisma: field,
			name_snake: format_ident!("{}", field.name().to_case(Case::Snake)),
			name_pascal: format_ident!("{}", field.name().to_case(Case::Pascal)),
			typ: FieldType::from(field),
		})
	}
}

impl<'a> Deref for Field<'a> {
	type Target = dml::Field;
	fn deref(&self) -> &Self::Target {
		&self.prisma
	}
}

#[derive(Debug)]
pub enum FieldType<'a> {
	Scalar {
		relations: RefCell<Option<Vec<Rc<Field<'a>>>>>,
	},
	Relation,
}

impl<'a> From<&dml::Field> for FieldType<'a> {
	fn from(field: &dml::Field) -> Self {
		match field.field_type() {
			dml::FieldType::Scalar(_, _, _) => FieldType::Scalar {
				relations: RefCell::new(None),
			},
			dml::FieldType::Relation(_) => FieldType::Relation,
			t => unimplemented!("Unsupported field type: {:?}", t),
		}
	}
}
