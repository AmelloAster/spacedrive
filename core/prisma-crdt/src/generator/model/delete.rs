use crate::generator::prelude::*;

pub fn generate(model: &Model) -> TokenStream {
	let Model {
		name_snake: model_name_snake,
		..
	} = model;

	quote! {
		pub struct Delete<'a> {
			client: &'a _prisma::PrismaCRDTClient,
			where_param: crate::prisma::#model_name_snake::UniqueWhereParam,
			with_params: Vec<crate::prisma::#model_name_snake::WithParam>,
		}

		impl<'a> Delete<'a> {
    		pub fn with(mut self, param: impl Into<crate::prisma::location::WithParam>) -> Self {
    			self.with_params.push(param.into());
    			self
    		}

    		pub async fn exec(self) -> Result<Option<crate::prisma::#model_name_snake::Data>, crate::prisma::QueryError> {

    		}
		}
	}
}
