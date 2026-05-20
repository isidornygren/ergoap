use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::Span;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, parse_macro_input};

fn generate_trait_registration(
    name: &Ident,
    register_call: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let crate_path = match crate_name("ergoap").expect("ergoap is not present") {
        FoundCrate::Itself => quote!(ergoap),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(#ident)
        }
    };

    quote! {
        impl RegisterComponentAs for #name {
            fn __register_as(world: &mut #crate_path::__macro_exports::bevy_ecs::world::World) {
                use #crate_path::__macro_exports::bevy_trait_query::RegisterExt;
                #register_call
            }
        }

        #crate_path::__macro_exports::inventory::submit! {
            AutomaticTraitRegistrations(
                <#name as RegisterComponentAs>::__register_as
            )
        }
    }
}

fn generate_world_sensor_impl(
    name: &Ident,
    field_type: &syn::Type,
    value_getter: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote! {
        impl WorldSensor for #name {
            fn sensor_value(&self) -> SensorValue {
                #value_getter.into()
            }
        }
        impl WorldSensorValue<#field_type> for #name {
            fn value(&self) -> #field_type {
                #value_getter
            }
        }
        impl SensorEffect<#field_type> for #name {}
        impl SensorComparison<#field_type> for #name {}
    }
}

fn generate_scorer_impl(
    name: &Ident,
    value_getter: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote! {
        impl Scorer for #name {
            fn score(&self) -> Score {
                #value_getter.into()
            }
        }
    }
}

#[proc_macro_derive(WorldSensor, attributes(world_sensor))]
pub fn derive_world_sensor(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let world_sensor_impl = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let field_type = &fields.unnamed.first().unwrap().ty;
                generate_world_sensor_impl(name, field_type, quote!(self.0))
            }
            Fields::Named(fields) => {
                let field = fields
                    .named
                    .iter()
                    .find(|f| {
                        f.attrs
                            .iter()
                            .any(|attr| attr.path().is_ident("world_sensor"))
                    })
                    .expect("No field marked with #[world_sensor] attribute found");

                let field_name = field.ident.as_ref().unwrap();
                let field_type = &field.ty;

                generate_world_sensor_impl(name, field_type, quote!(self.#field_name))
            }
            _ => panic!(
                "WorldSensor requires either a newtype struct (single unnamed field) or named fields with #[world_sensor] attribute"
            ),
        },
        _ => panic!("WorldSensor can only be derived for structs"),
    };

    let registration = generate_trait_registration(
        name,
        quote! {
            world.register_component_as::<dyn WorldSensor, #name>();
        },
    );

    let expanded = quote! {
        #world_sensor_impl
        #registration
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Scorer)]
pub fn derive_scorer(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let scorer_impl = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                generate_scorer_impl(name, quote!(self.0))
            }
            Fields::Named(fields) => {
                let field = fields
                    .named
                    .iter()
                    .find(|f| f.attrs.iter().any(|attr| attr.path().is_ident("score")))
                    .expect("No field marked with #[score] attribute found");

                let field_name = field.ident.as_ref().unwrap();

                generate_scorer_impl(name, quote!(self.#field_name))
            }
            _ => panic!(
                "Scorer requires either a newtype struct (single unnamed field) or named fields with #[score] attribute"
            ),
        },
        _ => panic!("Scorer can only be derived for structs"),
    };

    let registration = generate_trait_registration(
        name,
        quote! {
            world.register_component_as::<dyn GoalProviderTrait, GoalProvider<#name>>();
        },
    );

    let expanded = quote! {
        #scorer_impl
        #registration
    };

    TokenStream::from(expanded)
}
