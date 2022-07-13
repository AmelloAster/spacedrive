use crate::generator::prelude::*;

pub fn generate(model: &Model) -> TokenStream {
	let required_scalar_fields = model
		.fields
		.iter()
		.filter(|field| field.is_scalar_field() && field.required_on_create());

	let required_create_params = required_scalar_fields.clone().map(|field| {
		let field_name_snake = &field.name_snake;

		let field_type = match field.field_type() {
			dml::FieldType::Scalar(_, _, _) => field.type_tokens(),
			dml::FieldType::Relation(info) => {
				let relation_model_snake = format_ident!("{}", info.to.to_case(Case::Snake));

				quote!(crate::prisma::#relation_model_snake::Link)
			}
			dml::FieldType::Enum(e) => {
				let enum_name_pascal = format_ident!("{}", e.to_case(Case::Pascal));

				quote!(super::#enum_name_pascal)
			}
			_ => todo!(),
		};

		quote!(#field_name_snake: #field_type)
	});

	let required_crdt_create_params = required_scalar_fields
		.clone()
		.filter(|f| !model.is_sync_id(f))
		.map(|field| {
			let field_type = field.crdt_type_tokens();
			let field_name_snake = &field.name_snake;

			quote!(#field_name_snake: #field_type)
		});

	quote! {
		#[derive(Clone)]
		pub struct CreateParams {
			pub _params: Vec<SetParam>,
			#(pub #required_create_params),*
		}

		#[derive(Clone, ::serde::Serialize, ::serde::Deserialize)]
		pub struct CRDTCreateParams {
			#[serde(default, skip_serializing_if = "Vec::is_empty", rename = "_")]
			pub _params: Vec<CRDTSetParam>,
			#[serde(rename="_id")]
			pub _sync_id: SyncID,
			#(pub #required_crdt_create_params),*
		}
	}
}
