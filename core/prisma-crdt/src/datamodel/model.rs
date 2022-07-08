use std::{ops::Deref, str::FromStr};

use crate::attribute::Attribute;
use crate::prelude::*;

#[derive(Debug)]
pub struct Model<'a> {
	pub prisma: &'a dml::Model,
	pub name_snake: Ident,
	pub typ: ModelType<'a>,
	pub fields: Vec<Rc<RefCell<Field<'a>>>>,
}

impl<'a> Model<'a> {
	pub fn new(model: &'a dml::Model) -> Result<Rc<Self>, String> {
		let crdt_attribute = model
			.documentation
			.as_ref()
			.map(Attribute::parse)
			.map(Result::unwrap)
			.unwrap();

		let fields = model.fields().map(Field::new).collect::<Vec<_>>();

		let typ = ModelType::from_attribute(&crdt_attribute, &fields, model)?;

		let model = Rc::new(Self {
			prisma: model,
			name_snake: format_ident!("{}", model.name.to_case(Case::Snake)),
			typ,
			fields,
		});

		for field in &model.fields {
			*field.borrow_mut().model.borrow_mut() = Rc::downgrade(&model);
		}

		Ok(model)
	}

	pub fn resolve_relations(&self, datamodel: &Datamodel<'a>) {
		self.fields
			.iter()
			.for_each(|f| f.borrow().resolve_relations(self, datamodel));
	}

	pub fn get_sync_id(&self, primary_key: &FieldRef<'a>) -> Option<&FieldRef<'a>> {
		match &self.typ {
			ModelType::Local { id } => id.get_sync_id(primary_key),
			ModelType::Owned { id, .. } => id.get_sync_id(primary_key),
			ModelType::Shared { id, .. } => id.get_sync_id(primary_key),
			ModelType::Relation { .. } => None,
		}
	}
}

impl<'a> Deref for Model<'a> {
	type Target = dml::Model;
	fn deref(&self) -> &Self::Target {
		&self.prisma
	}
}

#[derive(Debug)]
pub enum ModelType<'a> {
	Local {
		id: SyncIDMapping<'a>,
	},
	Owned {
		owner: FieldRef<'a>,
		id: SyncIDMapping<'a>,
	},
	Shared {
		id: SyncIDMapping<'a>,
		create: SharedCreateType,
	},
	Relation {
		item: FieldRef<'a>,
		group: FieldRef<'a>,
	},
}

impl<'a> ModelType<'a> {
	pub fn from_attribute(
		attribute: &Attribute,
		fields: &[FieldRef<'a>],
		model: &'a dml::Model,
	) -> Result<Self, String> {
		let ret = match attribute.name {
			"local" => {
				let id = SyncIDMapping::from_attribute(attribute, fields, model)?;

				ModelType::Local { id }
			}
			"owned" => {
				let id = SyncIDMapping::from_attribute(attribute, fields, model)?;

				let owner = attribute
					.field("owner")
					.ok_or(format!("Missing owner field"))
					.and_then(|owner| {
						fields
							.iter()
							.find(|f| f.borrow().name() == owner)
							.map(Clone::clone)
							.ok_or(format!("Unknown owner field {}", owner))
					})?;

				ModelType::Owned { id, owner }
			}
			"shared" => {
				let id = SyncIDMapping::from_attribute(attribute, fields, model)?;

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
						fields
							.iter()
							.find(|f| f.borrow().name() == item)
							.map(Clone::clone)
							.ok_or(format!("Unknown item field {}", item))
					})?;

				let group = attribute
					.field("group")
					.ok_or(format!("Missing group field"))
					.and_then(|group| {
						fields
							.iter()
							.find(|f| f.borrow().name() == group)
							.map(Clone::clone)
							.ok_or(format!("Unknown group field {}", group))
					})?;

				ModelType::Relation { item, group }
			}
			name => Err(format!("Invalid attribute type {name}"))?,
		};

		Ok(ret)
	}
}

#[derive(Debug)]
pub enum SyncIDMapping<'a> {
	Single {
		primary_key: FieldRef<'a>,
		sync_id: FieldRef<'a>,
	},
	Compound(Vec<(FieldRef<'a>, FieldRef<'a>)>),
}

impl<'a> SyncIDMapping<'a> {
	pub fn from_attribute(
		attribute: &Attribute,
		fields: &[FieldRef<'a>],
		model: &dml::Model,
	) -> Result<Self, String> {
		let primary_key = model
			.primary_key
			.as_ref()
			.ok_or(format!("Model {} has no primary key", model.name))?;

		attribute
			.field("id")
			.map(|field_str| {
				if primary_key.fields.len() != 1 {
					Err(format!(
						"Model {} has a primary key with {} fields",
						model.name,
						primary_key.fields.len()
					))?
				}

				let primary_key_field = fields
					.iter()
					.find(|f| f.borrow().name() == &primary_key.fields[0].name)
					.ok_or(&format!(
						"Failed to find field {}",
						&primary_key.fields[0].name
					))?;

				fields
					.iter()
					.find(|f| f.borrow().name() == field_str)
					.map(|f| Self::Single {
						primary_key: primary_key_field.clone(),
						sync_id: f.clone(),
					})
					.ok_or(format!("Model {} has no field {field_str}", model.name))
			})
			.unwrap_or_else(|| {
				primary_key
					.fields
					.iter()
					.map(|field| {
						let primary_key_field = fields
							.iter()
							.find(|f| f.borrow().name() == &field.name)
							.map(Clone::clone)
							.ok_or(format!("Model {} has no field {}", model.name, field.name))?;

						fields
							.iter()
							.find(|f| f.borrow().name() == &field.name) // TODO: support arrays
							.map(|sync_id_field| (primary_key_field.clone(), sync_id_field.clone()))
							.ok_or(format!(
								"Unknown field {} on model {}",
								field.name, model.name
							))
					})
					.collect::<Result<_, _>>()
					.map(Self::Compound)
			})
	}

	pub fn get_sync_id(&self, primary_key: &FieldRef<'a>) -> Option<&FieldRef<'a>> {
		match self {
			Self::Single {
				primary_key: pk,
				sync_id,
			} if Rc::ptr_eq(primary_key, pk) => Some(sync_id),
			Self::Compound(mappings) => mappings
				.iter()
				.find(|(pk, _)| Rc::ptr_eq(primary_key, pk))
				.map(|(_, sync_id)| sync_id),
			_ => None,
		}
	}
}

#[derive(Debug)]
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
