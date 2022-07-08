use std::{ops::Deref, rc::Weak};

use crate::prelude::*;

pub type FieldRef<'a> = Rc<RefCell<Field<'a>>>;

#[derive(Debug)]
pub struct Field<'a> {
	pub prisma: &'a dml::Field,
	pub model: RefCell<Weak<Model<'a>>>,
	pub name_snake: Ident,
	pub name_pascal: Ident,
	pub typ: FieldType<'a>,
}

impl<'a> Field<'a> {
	pub fn resolve_relations(&self, model: &Model<'a>, datamodel: &Datamodel<'a>) {
		match &self.typ {
			FieldType::Scalar {
				relation_field_info: sync_relation,
			} => {
				*sync_relation.borrow_mut() = model
					.fields
					.iter()
					.find_map(|relation_field| {
						relation_field
							.borrow()
							.as_relation_field()
							.and_then(|relation_field_data| {
								// Finds position of scalar field in list of foreign keys of the relation
								relation_field_data
									.relation_info
									.fields
									.iter()
									.position(|rf_name| rf_name == self.name())
									.map(|pos| (relation_field_data, pos))
							})
							.and_then(|(relation_field_data, i)| {
								datamodel
									.models
									.iter()
									.find(|relation_model| {
										// Finds the model that the relation points to
										&relation_model.name
											== &relation_field_data.relation_info.to
									})
									.and_then(|relation_model| {
										// Finds the corresponding foreign key on the related model
										let ret =
											relation_model.fields.iter().find(|referenced_field| {
												referenced_field.borrow().name()
													== relation_field_data.relation_info.references
														[i]
											});

										ret
									})
							})
							.map(|referenced_field| {
								(relation_field.clone(), referenced_field.clone())
							})
					})
					.map(|(rel, ref_field)| RelationFieldInfo::new(rel, ref_field));
			}
			_ => {}
		}
	}

	pub fn new(field: &'a dml::Field) -> Rc<RefCell<Self>> {
		Rc::new(RefCell::new(Self {
			prisma: field,
			model: RefCell::new(Weak::new()),
			name_snake: format_ident!("{}", field.name().to_case(Case::Snake)),
			name_pascal: format_ident!("{}", field.name().to_case(Case::Pascal)),
			typ: FieldType::from(field),
		}))
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
		relation_field_info: RefCell<Option<RelationFieldInfo<'a>>>,
	},
	Relation,
}

impl<'a> From<&dml::Field> for FieldType<'a> {
	fn from(field: &dml::Field) -> Self {
		match field.field_type() {
			dml::FieldType::Scalar(_, _, _) => FieldType::Scalar {
				relation_field_info: RefCell::new(None),
			},
			dml::FieldType::Relation(_) => FieldType::Relation,
			t => unimplemented!("Unsupported field type: {:?}", t),
		}
	}
}

#[derive(Debug)]
pub struct RelationFieldInfo<'a> {
	pub relation: FieldRef<'a>,
	pub referenced_field: FieldRef<'a>,
}

impl<'a> RelationFieldInfo<'a> {
	pub fn new(relation: FieldRef<'a>, referenced_field: FieldRef<'a>) -> Self {
		Self {
			relation,
			referenced_field,
		}
	}
}
