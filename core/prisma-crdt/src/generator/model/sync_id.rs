use crate::generator::prelude::*;

pub fn generate(model: &Model) -> TokenStream {
	let sync_id_fields = model
		.fields
		.iter()
		.filter(|f| model.is_sync_id(f))
		.map(|field| {
            let field_type = field.crdt_type_tokens();
            let field_name_snake = &field.name_snake;
            
            quote!(#field_name_snake: #field_type)
        });

	quote! {
		#[derive(Clone, ::serde::Serialize, ::serde::Deserialize)]
		pub struct SyncID {
			#(pub #sync_id_fields),*
		}
	}
}
