mod actions;
mod set_param;
mod create;

use crate::generator::prelude::*;

pub fn generate(model: &Model, datamodel: &Datamodel) -> TokenStream {
	let Model { name_snake, .. } = model;

	let set_param_enums = set_param::generate(model);
	let actions_struct = actions::generate(model);
	let create_structs = create::generate(model, datamodel);

	quote!(
		pub mod #name_snake {
			#set_param_enums
			
			#create_structs

			#actions_struct
		}
	)
}
