use std::{ops::Deref, str::FromStr};

use crate::attribute::Attribute;
use crate::prelude::*;

pub struct Model<'a> {
	pub prisma: &'a dml::Model,
	pub name_snake: Ident,
	pub typ: ModelType<'a>,
	pub fields: Vec<Rc<Field<'a>>>,
}

impl<'a> Model<'a> {
	pub fn new(model: &'a dml::Model) -> Result<Rc<Self>, String> {
		let crdt_attribute = model
			.documentation
			.as_ref()
			.map(Attribute::parse)
			.map(Result::unwrap)
			.unwrap();

		let typ = ModelType::from_attribute(&crdt_attribute, model)?;

		let fields = model.fields().map(Field::new).collect::<Vec<_>>();

		let model = Rc::new(Self {
			prisma: model,
			name_snake: format_ident!("{}", model.name.to_case(Case::Snake)),
			typ,
			fields,
		});

		Ok(model)
	}

	pub fn resolve_relations(&self, datamodel: &Datamodel<'a>) {
		self.fields
			.iter()
			.for_each(|f| f.resolve_relations(datamodel));
	}
}

impl<'a> Deref for Model<'a> {
	type Target = dml::Model;
	fn deref(&self) -> &Self::Target {
		&self.prisma
	}
}

pub enum ModelType<'a> {
	Local {
		id: SyncIDField<'a>,
	},
	Owned {
		owner: &'a dml::Field,
		id: SyncIDField<'a>,
	},
	Shared {
		id: SyncIDField<'a>,
		create: SharedCreateType,
	},
	Relation {
		item: &'a dml::Field,
		group: &'a dml::Field,
	},
}

impl<'a> ModelType<'a> {
	pub fn from_attribute(attribute: &Attribute, model: &'a dml::Model) -> Result<Self, String> {
		let ret = match attribute.name {
			"local" => {
				let id = SyncIDField::from_attribute(attribute, model)?;

				ModelType::Local { id }
			}
			"owned" => {
				let id = SyncIDField::from_attribute(attribute, model)?;

				let owner = attribute
					.field("owner")
					.ok_or(format!("Missing owner field"))
					.and_then(|owner| {
						model
							.find_field(owner)
							.ok_or(format!("Unknown owner field {}", owner))
					})?;

				ModelType::Owned { id, owner }
			}
			"shared" => {
				let id = SyncIDField::from_attribute(attribute, model)?;

				let create = attribute
					.field("create")
					.map(SharedCreateType::from_str)
					.unwrap_or(Ok(SharedCreateType::Unique))?;

				ModelType::Shared { id, create }
			}
			"relation" => {
				let item = attribute
					.field("item")
					.ok_or(format!("Missing item field"))
					.and_then(|item| {
						model
							.find_field(item)
							.ok_or(format!("Unknown item field {}", item))
					})?;

				let group = attribute
					.field("group")
					.ok_or(format!("Missing group field"))
					.and_then(|group| {
						model
							.find_field(group)
							.ok_or(format!("Unknown group field {}", group))
					})?;

				ModelType::Relation { item, group }
			}
			name => Err(format!("Invalid attribute type {name}"))?,
		};

		Ok(ret)
	}
}

pub enum SyncIDField<'a> {
	Single(&'a dml::Field),
	Compound(Vec<&'a dml::Field>),
}

impl<'a> SyncIDField<'a> {
	pub fn from_attribute(attribute: &Attribute, model: &'a dml::Model) -> Result<Self, String> {
		attribute
			.field("id")
			.map(|field_str| {
				model
					.find_field(field_str)
					.map(Self::Single)
					.ok_or(format!("Model {} has no field {field_str}", model.name))
			})
			.unwrap_or_else(|| {
				model
					.primary_key
					.as_ref()
					.ok_or(format!("Model {} has no primary key", model.name))?
					.fields
					.iter()
					.map(|field| {
						model.find_field(&field.name).ok_or(format!(
							"Unknown field {} on model {}",
							field.name, model.name
						))
					})
					.collect::<Result<_, _>>()
					.map(Self::Compound)
			})
	}
}

pub enum SharedCreateType {
	Unique,
	Atomic,
}

impl FromStr for SharedCreateType {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let ret = match s {
			"Unique" => SharedCreateType::Unique,
			"Atomic" => SharedCreateType::Atomic,
			s => Err(format!("Invalid create type {}", s))?,
		};

		Ok(ret)
	}
}
