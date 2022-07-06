use prisma_client_rust_sdk::prisma_datamodel::dml;

pub enum SharedCreateType {
	Unique,
	Atomic,
}

pub enum ModelType<'a> {
	Local {
		id: &'a dml::Field,
	},
	Owned {
		owner: &'a dml::Field,
		id: &'a dml::Field,
	},
	Shared {
		id: &'a dml::Field,
		create: SharedCreateType,
	},
	Relation {
        item: &'a dml::Field,
        group: &'a dml::Field
    },
}

pub struct Model<'a> {
	prisma: &'a dml::Model,
	typ: ModelType<'a>,
}
