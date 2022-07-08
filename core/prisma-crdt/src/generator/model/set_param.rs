use crate::generator::prelude::*;

struct SetParam {
	pub variant: TokenStream,
	pub into_match_arm: TokenStream,
	pub into_crdt_match_arm: TokenStream,
	pub from_pcr_set_impl: TokenStream,
}

impl SetParam {
	pub fn new(field: &Field, model: &Model) -> Self {
		let Model {
			name_snake: model_name_snake,
			..
		} = model;
		let Field {
			name_snake: field_name_snake,
			name_pascal: field_name_pascal,
			..
		} = field;

		let variant_name = format_ident!("Set{}", field_name_pascal);

		let variant = {
			let variant_type = field.type_tokens();
			quote!(#variant_name(#variant_type))
		};

		let into_match_arm = quote!(Self::#variant_name(v) => crate::prisma::#model_name_snake::#field_name_snake::set(v));

		let into_crdt_match_arm = {
			let ret = match &field.typ {
				FieldType::Scalar {
					relation_field_info: sync_id_field,
				} => sync_id_field
					.borrow()
					.as_ref()
					.map(|sync_relation| {
						let relation_field_name =
							&sync_relation.referenced_field.borrow().name_snake;

						quote! {{
							let res = client
								.#model_name_snake()
								.find_unique(crate::prisma::#model_name_snake::#field_name_snake::equals(v))
								.exec()
								.await
								.unwrap()
								.unwrap();

							CRDTSetParam::#variant_name(res.#relation_field_name)
						}}
					})
					.unwrap_or(quote!(CRDTSetParam::#variant_name(v))),
				_ => unreachable!("{:#?}", field.prisma),
			};

			quote!(Self::#variant_name(v) => #ret)
		};

		let from_pcr_set_impl = quote! {
			impl From<crate::prisma::#model_name_snake::#field_name_snake::Set> for SetParam {
				fn from(v: crate::prisma::#model_name_snake::#field_name_snake::Set) -> Self {
					Self::#variant_name(v.0)
				}
			}
		};

		SetParam {
			variant,
			into_match_arm,
			into_crdt_match_arm,
			from_pcr_set_impl,
		}
	}
}

struct CRDTSetParam {
	pub variant: TokenStream,
	pub into_match_arm: TokenStream,
}

impl CRDTSetParam {
	pub fn new(field: &Field, model: &Model) -> Self {
		let Model {
			name_snake: model_name_snake,
			..
		} = model;
		let Field {
			name_snake: field_name_snake,
			name_pascal: field_name_pascal,
			typ: field_type,
			..
		} = field;

		let relation_field_info = match field_type {
			FieldType::Scalar {
				relation_field_info,
			} => relation_field_info,
			_ => unreachable!("Cannot create CRDTSetParam from relation field!"),
		}
		.borrow();

		let variant_name = format_ident!("Set{}", field_name_pascal);

		let variant = {
			let variant_type = match relation_field_info.as_ref() {
				Some(relation_field_info) => {
					let referenced_field = relation_field_info.referenced_field.borrow();
					let relation_model = referenced_field.model.borrow().upgrade().unwrap();

					let sync_id_field =
						relation_model.get_sync_id(&relation_field_info.referenced_field);

					match sync_id_field {
						Some(field) => field.borrow().type_tokens(),
						None => quote!(),
					}
				}
				None => field.type_tokens(),
			};

			let field_name = field.name();

			quote! {
				#[serde(rename = #field_name)]
				#variant_name(#variant_type)
			}
		};

		let into_match_arm = {
			let ret = match relation_field_info.as_ref() {
				Some(sync_relation) => {
					let relation_name_snake = &sync_relation.relation.borrow().name_snake;
					let sync_relation_referenced_field = sync_relation.referenced_field.borrow();
					let relation_model_name_snake = &sync_relation_referenced_field
						.model
						.borrow()
						.upgrade()
						.unwrap()
						.name_snake;
					let sync_id_field_name_snake = &sync_relation_referenced_field.name_snake;

					quote!(crate::prisma::#model_name_snake::#relation_name_snake::link(
						crate::prisma::#relation_model_name_snake::#sync_id_field_name_snake::equals(v)
					))
				}
				None => {
					quote!(crate::prisma::#model_name_snake::#field_name_snake::set(v))
				}
			};
			quote!(Self::#variant_name(v) => #ret)
		};

		Self {
			variant,
			into_match_arm,
		}
	}
}

pub fn generate(model: &Model, datamodel: &Datamodel) -> TokenStream {
	let Model {
		name_snake: model_name_snake,
		..
	} = model;

	let set_params = model
		.fields
		.iter()
		.filter(|f| f.borrow().is_scalar_field())
		.map(|f| SetParam::new(&*f.borrow(), model));

	let set_param_variants = set_params.clone().map(|p| p.variant);
	let set_param_into_match_arms = set_params.clone().map(|p| p.into_match_arm);
	let set_param_into_crdt_match_arms = set_params.clone().map(|p| p.into_crdt_match_arm);
	let set_param_from_pcr_set_impls = set_params.clone().map(|p| p.from_pcr_set_impl);

	let crdt_set_params = model
		.fields
		.iter()
		.filter(|f| f.borrow().is_scalar_field())
		.map(|f| CRDTSetParam::new(&*f.borrow(), model));

	let crdt_set_param_variants = crdt_set_params.clone().map(|p| p.variant);
	let crdt_set_param_into_match_arms = crdt_set_params.clone().map(|p| p.into_match_arm);

	quote! {
		#[derive(Clone)]
		pub enum SetParam {
			#(#set_param_variants),*
		}

		impl SetParam {
			pub async fn into_crdt(self) -> CRDTSetParam {
				match self {
					#(#set_param_into_crdt_match_arms),*
				}
			}
		}

		#(#set_param_from_pcr_set_impls)*

		impl Into<crate::prisma::#model_name_snake::SetParam> for SetParam {
			fn into(self) -> crate::prisma::#model_name_snake::SetParam {
				match self {
					#(#set_param_into_match_arms),*
				}
			}
		}

		#[derive(Clone, serde::Serialize, serde::Deserialize)]
		pub enum CRDTSetParam {
			#(#crdt_set_param_variants),*
		}

		impl Into<crate::prisma::#model_name_snake::SetParam> for CRDTSetParam {
			fn into(self) -> crate::prisma::#model_name_snake::SetParam {
				match self {
					#(#crdt_set_param_into_match_arms),*
				}
			}
		}
	}
}
