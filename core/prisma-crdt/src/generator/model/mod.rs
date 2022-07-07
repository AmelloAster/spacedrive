mod actions;
mod set_param;

use crate::generator::prelude::*;

pub fn generate(model: &Model, datamodel: &Datamodel) -> TokenStream {
	let Model { name_snake, .. } = model;

	let set_param_enums = set_param::generate(model, datamodel);
	let actions_struct = actions::generate(model);

	quote!(
		pub mod #name_snake {
			#set_param_enums

			#actions_struct
		}
	)
}
