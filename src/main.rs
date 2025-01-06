use bevy::{prelude::*,input::*};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;
use std::thread;
use std::sync::Mutex;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
const MAX_AMPLITUDE_F32: f32 = (u16::MAX / 2) as f32;
const MIN_DB: f32 = -96.0;
static mut DECIBEL: Mutex<f32> = Mutex::new(0.0);

#[derive(Resource)]
pub struct UiStufff{
    thresh: f32,
    path1: String,
    path2: String,
    color: Srgba,
    bro_size: f32,
    inactiveopac:f32,
    show_tuto:bool,
    show_window: bool,
    distance: f32,
    xpos: f32,
    ypos: f32,
    flip: bool
}
#[derive(Resource)]
pub struct State {
    is_active: bool,
}
#[derive(Resource)]
struct EnId {
    cam: Entity,
    en1: Entity,
    en2: Entity,
}
#[derive(Resource)]
struct MeterdB(f32);



fn main() {
    thread::spawn(move || {
        start_audio_stream();
    });

    App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(EguiPlugin)
    .insert_resource(State { is_active: true })
    .insert_resource(EnId {
        cam: Entity::from_raw(0),
        en1: Entity::from_raw(0),
                     en2: Entity::from_raw(0),
    })
    .insert_resource(MeterdB(0.0))
    .insert_resource(ClearColor(Color::rgba(0.0, 1.0, 0.0,0.0)))
    .insert_resource(UiStufff{
        thresh: -5.0,
        path1: "../Idle.png".to_string(),
        path2: "../Active.png".to_string(),
        color: Srgba::new(0.0,0.0 ,0.0 , 1.0),
        bro_size: 200.0,
        inactiveopac: 0.5,
        show_tuto: false,
        show_window: true,
        distance: 0.0 ,
        xpos: 0.0,
        ypos: 0.0,
        flip: false

    })
    .add_systems(Startup,setup)
    .add_systems(Update,(action,ui_system,dealcibel))
    .run();
}

fn ui_system(
    mut contexts: EguiContexts,
    mut uist: ResMut<UiStufff>,
    mut clear_color: ResMut<ClearColor>,
    mdb: Res<MeterdB>,
) {
    let (
        mut red,
        mut green,
        mut blue,
        mut size,
        mut show_tuto,
        mut thresh,
        mut distance,
         mut xpos,
         mut ypos,
         mut flip,
         mut inactiveopac
    ) = (
        uist.color.red * 255.0,
        uist.color.green * 255.0,
        uist.color.blue * 255.0,
        uist.bro_size,
        uist.show_tuto,
        uist.thresh,
        uist.distance,
        uist.xpos,
        uist.ypos,
         uist.flip,
         uist.inactiveopac*100.0,
    );

    if uist.show_window {
        egui::Window::new("Aura Configs").show(contexts.ctx_mut(), |ui| {
            // Sliders for controlling various settings
            ui.label(format!("{} dB",mdb.0));
            ui.add(egui::Slider::new(&mut thresh, -100.0..=20.0).text("Voice lvl threshold").step_by(0.1));
            ui.add(egui::Slider::new(&mut size, 0.0..=1000.0).text("Size"));
            ui.add(egui::Slider::new(&mut inactiveopac, 0.0..=100.0).text("Dimness"));
            ui.add(egui::Slider::new(&mut distance, 0.0..=200.0).text("Bounce"));
            ui.add(egui::Slider::new(&mut xpos, -500.0..=500.0).text("X"));
            ui.add(egui::Slider::new(&mut ypos, -500.0..=500.0).text("Y"));
            ui.add(egui::Slider::new(&mut red, 0.0..=255.0).text("Red"));
            ui.add(egui::Slider::new(&mut green, 0.0..=255.0).text("Green"));
            ui.add(egui::Slider::new(&mut blue, 0.0..=255.0).text("Blue"));


            if ui.button("Flip").clicked() {
                flip = !flip;
            }

            if ui.button(if show_tuto { "Hide Yap" } else { "Show Yap" }).clicked() {
                show_tuto = !show_tuto;
            }

            if show_tuto {
                ui.label("Put the images in this directory!!");
                ui.label("With the name Idle.png and Active.png");
                ui.label("Why? Cuz sigma.");
            }

            if ui.button("Close Window (TAB)").clicked() {
                uist.show_window = false;
            }

            if ui.button("Go Default").clicked() {
                (
                    red,
                 green,
                 blue,
                 size,
                 show_tuto,
                 thresh,
                 distance,
                 xpos,
                 ypos,
                 flip,
                 inactiveopac,
                ) = (
                    0.0,
                     0.0,
                     0.0,
                     200.0,
                     show_tuto,
                     -5.0,
                     0.0,
                     0.0,
                     0.0,
                     false,
                     50.0,

            );}


            (
                uist.color.red,
                 uist.color.green,
                 uist.color.blue,
                 uist.bro_size,
                 uist.show_tuto,
                 uist.thresh,
                 uist.distance,
             uist.xpos,
             uist.ypos,
             uist.flip,
             uist.inactiveopac,

            ) = (
                red / 255.0,
                 green / 255.0,
                 blue / 255.0,
                 size,
                 show_tuto,
                 thresh,
                 distance,
                 xpos,
                 ypos,
                 flip,
                 inactiveopac/100.0,

            );

            clear_color.0 = Color::srgba(
                uist.color.red,
                uist.color.green,
                uist.color.blue,
                1.0,
            );
        });
    }
}


fn start_audio_stream() {
    let host = cpal::default_host();
    let device = host.default_input_device().expect("No input device available");
    let config = device.default_input_config().expect("No input config available");

    let stream = match config.sample_format() {
        SampleFormat::F32 => device.build_input_stream(
            &config.into(),
                                                       data_callback,
                                                       |e| eprintln!("5Error: {:?}", e),
                                                       None,
        ),
        _ => panic!("Unsupported sample format"),
    }
    .expect("Failed to build input stream");

    stream.play().expect("Failed to start stream");

    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn db_fs(data: &[f32]) -> f32 {
    let max = data.iter()
    .map(|f| (*f * i16::MAX as f32).abs() as u16)
    .max()
    .unwrap_or(0);

    (20.0f32 * (max as f32 / MAX_AMPLITUDE_F32).log10()).clamp(MIN_DB, 20.0)
}

fn data_callback(data: &[f32], _info: &cpal::InputCallbackInfo) {
    let db = db_fs(data);
    unsafe{
        let mut decibel = DECIBEL.lock().unwrap();
        *decibel = db;
    }

}

fn dealcibel(
    mut mdb : ResMut<MeterdB>
){

unsafe {
    let decibel_value = DECIBEL.lock().unwrap();
    mdb.0 = *decibel_value;

    }
}

fn setup(
    mut commands: Commands,
    ast_ldr: Res<AssetServer>,
    mut enid: ResMut<EnId>,
    uist: Res<UiStufff>,

) {
    enid.cam = commands.spawn(Camera2d).id();

    enid.en1 = commands.spawn(Sprite {
        image: ast_ldr.load(uist.path1.clone()),
                              custom_size: Some(Vec2::splat(200.0)),
                              ..default()
    })
    .id();

    enid.en2 = commands.spawn(Sprite {
        image: ast_ldr.load(uist.path2.clone()),
                              custom_size: Some(Vec2::splat(200.0)),
                              ..default()
    })
    .id();
}

fn action(
    mut commands: Commands,
    inp: Res<ButtonInput<KeyCode>>,
    enid: Res<EnId>,
    ast_ldr: Res<AssetServer>,
    mdb : ResMut<MeterdB>,
    mut state: ResMut<State>,
    mut uist: ResMut<UiStufff>,
) {

    state.is_active = mdb.0 >= uist.thresh    ;

    if inp.just_pressed(KeyCode::Tab) {
        uist.show_window = !uist.show_window;
    }

    let opac = uist.inactiveopac;
    commands.entity(enid.en1)
    .insert(Sprite {
        color: if state.is_active { Color::srgba(1.0-opac, 1.0-opac, 1.0-opac, 0.0) } else { Color::srgba(1.0-opac, 1.0-opac, 1.0-opac, 1.0) },
                                     image: ast_ldr.load(uist.path1.clone()),
                                     custom_size: Some(Vec2::splat(uist.bro_size)),
                                        flip_x: uist.flip,
                                     ..default()
    })
    .insert(Transform{
        translation: Vec3::new(uist.xpos, uist.ypos, 0.0),
            ..default()
    });

    commands.entity(enid.en2)
    .insert(Sprite {
        color: if state.is_active { Color::srgba(1.0, 1.0, 1.0, 1.0) } else { Color::srgba(1.0, 1.0, 1.0, 0.0) },
                                     image: ast_ldr.load(uist.path2.clone()),
                                     custom_size: Some(Vec2::splat(uist.bro_size)),
                                    flip_x: uist.flip,
                                     ..default()
    })
    .insert(Transform{
        translation: Vec3::new(uist.xpos, uist.ypos+uist.distance, 0.0),
            ..default()
    });

}

