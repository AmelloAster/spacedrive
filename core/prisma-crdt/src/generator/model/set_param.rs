use crate::generator::prelude::*;

struct SetParam {
	pub variant: TokenStream,
	pub into_match_arm: TokenStream,
	pub into_crdt_match_arm: TokenStream,
}

impl SetParam {
	pub fn new(model: &Model, field: &Field) -> Self {
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
				FieldType::Scalar { relations } => relations
					.borrow()
					.as_ref()
					.map(|relations| {
						let relation_field_name = &relations[0].name_snake;

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

		SetParam {
			variant,
			into_match_arm,
			into_crdt_match_arm,
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
		.filter_map(|f| f.is_scalar_field().then(|| SetParam::new(model, f)));

	let set_param_variants = set_params.clone().map(|p| p.variant);
	let set_param_into_match_arms = set_params.clone().map(|p| p.into_match_arm);
	let set_param_into_crdt_match_arms = set_params.clone().map(|p| p.into_crdt_match_arm);

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

		impl Into<crate::prisma::#model_name_snake::SetParam> for SetParam {
			fn into(self) -> crate::prisma::#model_name_snake::SetParam {
				match self {
					#(#set_param_into_match_arms),*
				}
			}
		}

		// #[derive(Clone, serde::Serialize, serde::Deserialize)]
		// pub enum CRDTSetParam {

		// }

		// impl Into<crate::prisma::#model_name_snake::SetParam> for CRDTSetParam {
		// 	fn into(self) -> crate::prisma::#model_name_snake::SetParam {
		// 		match self {

		// 		}
		// 	}
		// }
	}
}
