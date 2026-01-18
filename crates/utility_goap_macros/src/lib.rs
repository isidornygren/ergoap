use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(WorldSensor, attributes(world_sensor))]
pub fn derive_world_sensor(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

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
    };

    TokenStream::from(expanded)
}
