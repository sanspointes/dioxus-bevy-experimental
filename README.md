# Bevy (Sanspointes) Dioxus Experiment

<div align="center">
  <h3 align="center">diouxs_bevy</h3>

  <p align="center">
    Experimental, extensible Bevy renderer for Dioxus.
  </p>
</div>

> A lot of this is based off [Jasmine's bevy_dioxus example](https://github.com/JMS55/bevy_dioxus/).  Thanks
> for doing most of the work.

This is pre-alpha software and will either be heavily refactored (to make defining all of your elements less verbose) or abandoned. 

## How (do I use it)?

```rust
// You're going to have to define all of your elements / attributes in here.
#[bevy_spts_dioxus]
pub mod my_adapter {

    // Implement custom attributes
    #[define_attr]
    pub fn is_visible_attr(world: &mut World, entity: Entity, value: AttributeValue) {
        let mut entity_mut = world.entity_mut(entity);
        let is_visible = value.as_bool().unwrap_or(false);
        entity_mut.get_mut::<Visibility>() = if is_visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    pub mod dioxus_elements {

        #[define_element]
        pub struct spatial {
            // Components will be spawned with the element.
            #[component]
            transform: Transform,
            #[component]
            visibility: Visibility,

            // Can compose your attributes across multiple elements. 
            #[attr]
            is_visible: is_visible_attr,
        }

    }
}

#[component]
pub fn root_component() -> Element {
    rsx! {
        spatial {
            // Pass your values to your attributes
            is_visible: true,
            // Reactively set whole attributes (must be wrapped with the WA, WrappedAttribute, struct)
            transform: WA(Transform::from_xyz(0., 5., 0.5)),

            // Only dependency is bevy_hierarchy
            spatial {
                visibility: WA(Visibility::Visible),
            }
        }
    }
}

pub fn spawn_root(mut commands: Commands) {
    commands.spawn((Transform::default(), Visibility::default(), SptsDioxusRootComponent(root)));
}

pub fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(SptsDioxusPlugin::<my_adapter::SptsDioxusAdapter>::default());
    // Spawn your root bundle.
    app.add_systems(Startup, spawn_root);
}

```

## How (does it work)?

The code in [bevy-spts-dioxus-core](./bevy-spts-dioxus-core/) is generic over a 
`SptsDioxusTemplateNode` trait.  This trait implements all of the 
behaviour specific to your app, such as defining all of your elements,
how to spawn your elements, and how to apply the attributes.

When you define your adapter module with `#[bevy_spts_dioxus]` it will generate 
a `SptsDioxusAdapter` struct within that module that implements the `SptsDioxusTemplateNode`
trait.  If you expand the macro it is (at least in this simple, early stage), 
not too hard to see how it all works.

The `SptsDioxusAdapter` is actually an enum of all of your different element types.

## Gotchas 

### Global attribute keys
Currently all attributes (whether they're components or from `#[define_attr]`)
need to have a globally unique key, two elements can't have different attributes
under the same key.  I'm open to fixing this but it's low priority.

### Element naming
Elements must be lowercase + not use any semi-colons.  This is a carry over from dioxus.
If you try to name your element `mesh_3d` you'll get a not so nice error.  Instead you'll
have to name it `mesh3d`.

## What next

Firstly, not sure of the life-span of this project.  If this experiment is promising and I can make use of it 
for my own projects I'll try to invest more time into it.

- [ ] Cleanup logs and warnings
- [ ] Figure out what dioxus `Dynamic` nodes are for?  The don't seem to effect the heirarchy, is it possible we can just ignore them? 
- [ ] Figure out a good API for defining event listeners. 
- [ ] Build up a library of `macro_rules` to help cut down on boilerplate, i.e. provide `include_spatial_attr_definitions`, `include_spatial_components`, `include_spatial_attrs` that can be slotted into your `#[bevy_spts_dioxus]` module.
- [ ] Add necessary hooks
    - [x] `Hooks::use_world_memo()` - Use memo with access to world 
    - [x] `Hooks::use_world_callback()` - Use callback with access to world.
- [ ] Either fix the [global attribute keys](#global-attribute-keys) gotcha or provide a better error message.
- [ ] Better error message when [element has a bad name](#element-naming).
