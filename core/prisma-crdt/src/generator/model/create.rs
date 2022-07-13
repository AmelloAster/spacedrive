use crate::generator::prelude::*;

pub fn generate(model: &Model) -> TokenStream {
	let Model {
		name_snake: model_name_snake,
		..
	} = model;

	quote! {
		pub struct Create<'a> {
			client: &'a super::_prisma::PrismaCRDTClient,
			set_params: CreateParams,
			with_params: Vec<crate::prisma::#model_name_snake::WithParam>,
		}

		impl<'a> Create<'a> {
			pub fn with(mut self, param: impl Into<crate::prisma::#model_name_snake::WithParam>) -> Self {
				self.with_params.push(param.into());
				self
			}

			pub async fn exec(self) -> Result<crate::prisma::#model_name_snake::Data, crate::prisma::QueryError> {

			}
		}
	}
}
