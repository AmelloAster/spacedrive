use crate::generator::prelude::*;

pub fn constructor<'a>(
	model: &'a Model,
	data_var: TokenStream,
	datamodel: &'a Datamodel,
) -> TokenStream {
	let args = model
		.fields
		.iter()
		.filter(|f| model.is_sync_id(f.name()))
		.flat_map(|f| match &f.typ {
			FieldType::Scalar { .. } => vec![f],
			FieldType::Relation { relation_info } => relation_info
				.fields
				.iter()
				.map(|f| {
					model
						.field(f)
						.expect(&format!("{} has no field {}", model.name, f))
				})
				.collect(),
		})
		.map(|f| {
			let field_name_snake = snake_ident(f.name());

			let val = match &f.typ {
				FieldType::Scalar {
					relation_field_info,
				} => match relation_field_info {
					Some(relation_field_info) => {
						let relation_model_name_snake =
							snake_ident(relation_field_info.referenced_model);
						let relation_model = datamodel
							.model(relation_field_info.referenced_model)
							.unwrap();
						let relation_field_name_snake =
							snake_ident(relation_field_info.referenced_field);

						let referenced_field_name_snake = format_ident!(
							"{}",
							relation_model
								.sync_id_for_pk(&relation_field_info.referenced_field)
								.map(|f| f.name())
								.unwrap_or(&relation_field_info.referenced_field)
						);

						quote!({
							#[doc = "TODO: fetch from cache"]
							self
								.client
								.client
								.#relation_model_name_snake()
								.find_unique(crate::prisma::#relation_model_name_snake::#relation_field_name_snake::equals(
									#data_var.#field_name_snake.clone()
								))
								.exec()
								.await?
								.unwrap()
								.#referenced_field_name_snake
						})
					}
					None => quote!(#data_var.#field_name_snake.clone()),
				},
				_ => unreachable!(),
			};

			quote!(#field_name_snake: #val)
		});

	quote! {
		SyncID {
			#(#args,)*
		}
	}
}

pub fn definition(model: &Model, datamodel: &Datamodel) -> TokenStream {
	let sync_id_fields = model.scalar_sync_id_fields(datamodel).map(|field| {
		let field_type = field.1.crdt_type_tokens(datamodel);
		let field_name_snake = snake_ident(field.0.name());

		quote!(#field_name_snake: #field_type)
	});

	quote! {
		#[derive(Clone, ::serde::Serialize, ::serde::Deserialize)]
		pub struct SyncID {
			#(pub #sync_id_fields),*
		}
	}
}
