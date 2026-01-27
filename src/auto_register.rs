use bevy_ecs::world::World;

pub struct AutomaticTraitRegistrations(pub fn(&mut World));

pub fn register_trait_types(registry: &mut World) {
    for registration_fn in inventory::iter::<AutomaticTraitRegistrations> {
        registration_fn.0(registry);
    }
}

pub trait RegisterComponentAs {
    fn __register_as(world: &mut World);
}

inventory::collect!(AutomaticTraitRegistrations);
