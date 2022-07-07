mod field;
mod model;

pub use field::*;
pub use model::*;

use crate::prelude::*;

pub const INTERNAL_MODELS: &'static [&'static str] =
	&["OwnedOperation", "SharedOperation", "RelationOperation"];

pub struct Datamodel<'a> {
	pub prisma: &'a dml::Datamodel,
	pub models: Vec<Rc<Model<'a>>>,
}

impl<'a> Datamodel<'a> {
	pub fn resolve_relations(&self) {
		self.models.iter().for_each(|m| m.resolve_relations(self));
	}
}

impl<'a> TryFrom<&'a dml::Datamodel> for Datamodel<'a> {
	type Error = String;
	fn try_from(datamodel: &'a dml::Datamodel) -> Result<Self, Self::Error> {
		let models = datamodel
			.models
			.iter()
			.filter(|m| !INTERNAL_MODELS.contains(&m.name.as_str()))
			.map(Model::new)
			.collect::<Result<Vec<_>, _>>()?;

		let datamodel = Self {
			prisma: datamodel,
			models,
		};

		datamodel.resolve_relations();

		Ok(datamodel)
	}
}
