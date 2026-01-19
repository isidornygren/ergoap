use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::Span;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, parse_macro_input};

#[proc_macro_derive(WorldSensor, attributes(world_sensor))]
pub fn derive_world_sensor(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let crate_path = match crate_name("utility-goap").expect("utility-goap is not present") {
        FoundCrate::Itself => quote!(utility_goap),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!( #ident )
        }
    };

    let world_sensor_impl = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                quote! {
                    impl WorldSensor for #name {
                        fn sensor_value(&self) -> SensorValue {
                            self.0.into()
                        }
                    }
                }
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

                quote! {
                    impl WorldSensor for #name {
                        fn sensor_value(&self) -> SensorValue {
                            self.#field_name.into()
                        }
                    }
                }
            }
            _ => panic!(
                "WorldSensor requires either a newtype struct (single unnamed field) or named fields with #[world_sensor] attribute"
            ),
        },
        _ => panic!("WorldSensor can only be derived for structs"),
    };

    let expanded = quote! {
        #world_sensor_impl
        impl SensorComparison for #name {}
        impl SensorEffect for #name {}
        impl RegisterComponentAs for #name {
            fn __register_as(world: &mut #crate_path::__macro_exports::bevy_ecs::world::World) {
                use #crate_path::__macro_exports::bevy_trait_query::RegisterExt;
                world.register_component_as::<dyn WorldSensor, #name>();
            }
        }

        #crate_path::__macro_exports::inventory::submit! {
            AutomaticTraitRegistrations(
                <#name as RegisterComponentAs>::__register_as
            )
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Action)]
pub fn derive_action(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let crate_path = quote! { ::utility_goap };

    let expanded = quote! {
        impl RegisterComponentAs for #name {
            fn __register_as(world: &mut #crate_path::__macro_exports::bevy_ecs::world::World) {
                use #crate_path::__macro_exports::bevy_trait_query::RegisterExt;
                world.register_component_as::<dyn ActionProviderTrait, ActionProvider<CurrentAction<#name>>>();
            }
        }

        #crate_path::__macro_exports::inventory::submit! {
            AutomaticTraitRegistrations(
                <#name as RegisterComponentAs>::__register_as
            )
        }
    };

    TokenStream::from(expanded)
}
