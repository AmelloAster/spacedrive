use crate::generator::prelude::*;

pub fn find_accessors(name: &Ident) -> TokenStream {
	quote! {
		pub fn find_unique(
			self,
			param: crate::prisma::#name::UniqueWhereParam,
		) -> crate::prisma::#name::FindUnique<'a> {
			self.client.client.#name().find_unique(param)
		}

		pub fn find_many(
			self,
			params: Vec<crate::prisma::#name::WhereParam>,
		) -> crate::prisma::#name::FindMany<'a> {
			self.client.client.#name().find_many(params)
		}

		pub fn find_many(
			self,
			params: Vec<crate::prisma::#name::WhereParam>,
		) -> crate::prisma::#name::FindMany<'a> {
			self.client.client.#name().find_many(params)
		}
	}
}

pub fn generate(model: &Model) -> TokenStream {
	let Model { name_snake, .. } = model;

	let find_accessors = find_accessors(name_snake);

	quote! {
		pub struct Actions<'a> {
			client: &'a _prisma::PrismaCRDTClient,
		}

		impl<'a> Actions<'a> {
			pub(super) fn new(client: &super::PrismaCRDTClient) -> Self {
				Self { client }
			}

			#find_accessors

			pub fn update(
				self,
				_where: crate::prisma::#name_snake::UniqueWhereParam,
				set_params: Vec<SetParam>,
			) -> Update<'a> {
				Update {
					client: self.client,
					where_param: _where,
					set_params,
				}
			}

			pub fn delete(self, param: crate::prisma::#name_snake::UniqueWhereParam) -> Delete<'a> {
				Delete {
					client: self.client,
					r#where: param,
					with_params: vec![],
				}
			}
		}
	}
}
