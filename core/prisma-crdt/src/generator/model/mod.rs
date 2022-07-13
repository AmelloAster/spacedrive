mod actions;
mod create;
mod create_params;
mod delete;
mod set_param;
mod sync_id;
mod update;

use crate::generator::prelude::*;

pub fn generate(model: &Model, datamodel: &Datamodel) -> TokenStream {
	let Model { name_snake, .. } = model;

	let set_param_enums = set_param::generate(model);
	let sync_id_struct = sync_id::generate(model);
	let create_params = create_params::generate(model);

	let create_struct = create::generate(model);
	let update_struct = update::generate(model);
	let delete_struct = delete::generate(model);

	let actions_struct = actions::generate(model);

	quote!(
		pub mod #name_snake {
			#set_param_enums

			#sync_id_struct

			#create_params

			#create_struct

			#update_struct

			#delete_struct

			#actions_struct
		}
	)
}
