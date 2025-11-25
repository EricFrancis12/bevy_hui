use bevy::prelude::*;
use bevy_hui::prelude::*;
use bevy_hui_widgets::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin {
                default_sampler: bevy::image::ImageSamplerDescriptor::nearest(),
            }),
            HuiPlugin,
            HuiSliderWidgetPlugin,
            HuiInputWidgetPlugin,
            HuiSelectWidgetPlugin,
        ))
        .add_systems(
            Startup,
            (register_widgets, setup_scene, register_user_functions),
        )
        .add_systems(Update, (update_slider_target_text, handle_mouse_up))
        .add_observer(add_wants_fire_to_slider_mouse_up)
        .run();
}

#[derive(Component)]
pub struct WantsFire(pub bool);

fn add_wants_fire_to_slider_mouse_up(
    trigger: Trigger<OnAdd, OnUiChangeMouseUp>,
    mut commands: Commands,
) {
    let entity = trigger.target();
    commands.entity(entity).insert(WantsFire(false));
}

fn handle_mouse_up(
    mut commands: Commands,
    mut query: Query<(Entity, &OnUiChangeMouseUp, &mut WantsFire)>,
    buttons: Res<ButtonInput<MouseButton>>,
    function_bindings: Res<FunctionBindings>,
) {
    if buttons.just_released(MouseButton::Left) {
        for (entity, funcs, mut wants_fire) in query.iter_mut() {
            for fn_str in funcs.iter() {
                if !wants_fire.0 {
                    continue;
                }
                wants_fire.0 = false;
                function_bindings.maybe_run(fn_str, entity, &mut commands);
            }
        }
    }
}

fn register_widgets(mut html_comps: HtmlComponents, server: Res<AssetServer>) {
    html_comps.register("slider", server.load("widgets/my_slider.html"));
}

fn setup_scene(mut cmd: Commands, server: Res<AssetServer>) {
    cmd.spawn(Camera2d);
    cmd.spawn((
        HtmlNode(server.load("widgets/my_widget.html")),
        TemplateProperties::default().with("title", "Test-title"),
    ));
}

/// If you trigger the [bevy_hui::prelude::UiChangedEvent] in your widget
/// code, you can bind custom functions in html to this event.
fn register_user_functions(mut html_funcs: HtmlFunctions) {
    html_funcs.register(
        "notify_slider_change",
        |In(entity), mut sliders: Query<(&Slider, &mut WantsFire)>| {
            let Ok((slider, mut wants_fire)) = sliders.get_mut(entity) else {
                return;
            };
            if !wants_fire.0 {
                wants_fire.0 = true;
            }
            info!("Slider {entity} changed, new value: {:.2}", slider.value);
        },
    );

    html_funcs.register(
        "notify_slider_mouse_up",
        |In(entity), sliders: Query<&Slider>| {
            let Ok(slider) = sliders.get(entity) else {
                return;
            };
            info!("Slider mouse-up on {entity}. Value: {:?}", slider.value);
        },
    );

    html_funcs.register(
        "notify_input_change",
        |In(entity), sliders: Query<&TextInput>| {
            let Ok(input) = sliders.get(entity) else {
                return;
            };
            info!("Input {entity} changed, new value: `{}`", input.value);
        },
    );

    html_funcs.register(
        "notify_select_change",
        |In(entity), sliders: Query<&SelectInput>| {
            let Ok(select) = sliders.get(entity) else {
                return;
            };
            info!("Select {entity} changed, new value: {:?}", select.value);
        },
    );

    html_funcs.register("on_button_press", |In(entity)| {
        info!("Button pressed on entity: {entity}");
    });
}

// -----------------
// example, custom user extension, update a value display of a slider
fn update_slider_target_text(
    mut events: EventReader<SliderChangedEvent>,
    targets: Query<&UiTarget>,
    mut texts: Query<&mut Text>,
) {
    for event in events.read() {
        let Ok(target) = targets.get(event.slider) else {
            continue;
        };

        let Ok(mut text) = texts.get_mut(**target) else {
            continue;
        };

        text.0 = format!("{:.2}", event.value);
    }
}
