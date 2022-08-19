#![allow(unused)]

mod vector_field;

use itertools::Itertools;
use nannou::prelude::*;
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use crate::vector_field::VectorField;

pub struct Model {
    window: window::Id,
    pub version: usize,
    state: Box<State>,
}

impl Model {
    pub fn new(window: WindowId) -> Self {
        Self {
            window,
            version: 0,
            state: Box::new(State::default()),
        }
    }
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

#[derive(Debug, Default)]
struct Thing {
    pos: Vec2,
    color: Rgb,
    width: f32,
    age: u64,
}

impl Thing {
    pub fn random(bounds: Rect) -> Self {
        Self {
            pos: pt2(
                random_range(bounds.left(), bounds.right()),
                random_range(bounds.bottom(), bounds.top()),
            )
            .into(),
            color: rgb(random_f32(), random_f32(), random_f32()),
            width: random_range(1.0, 10.0),
            age: 1,
        }
    }

    fn pos(&self) -> Vec2 {
        self.pos
    }
}

#[derive(Default, Debug)]
pub struct State {
    version: usize,
    vector_field: VectorField<20, 20>,
    things: Vec<Thing>,
    needs_draw: bool,
    updates: usize,
    mouse_pos: Option<Vec2>,
}

#[no_mangle]
pub fn event(app: &App, model: &mut Model, event: WindowEvent) {}

#[no_mangle]
pub fn update(app: &App, model: &mut Model, update: Update) {
    let bounds = app.window_rect();

    const MAX_THINGS: usize = 500;

    if model.state.version < model.version || app.elapsed_frames() == 0 {
        println!("new version {}", model.version);
        model.state = Box::new(State {
            version: model.version,
            // vector_field: VectorField::new(bounds, |pos| pos.rotate(deg_to_rad(90.0))),
            vector_field: VectorField::new(bounds, |pos| {
                pos.rotate(deg_to_rad(random_range(-90.0, 90.0)))
            }),
            things: (0..MAX_THINGS).map(|_| Thing::random(bounds)).collect(),
            needs_draw: true,
            ..Default::default()
        });
    }

    let State {
        vector_field,
        things,
        needs_draw,
        updates,
        mouse_pos,
        ..
    } = &mut *model.state;

    // if *updates > 5 {
    //     *needs_draw = false;
    //     return;
    // }
    *updates += 1;

    *needs_draw = true;
    if let Some(pos) = mouse_pos.take() {
        vector_field.update(|pos2, vec| (pos - pos2));
    }

    vector_field.update(|pos2, vec| vec.rotate(deg_to_rad(1.0)));

    for thing in things.iter_mut() {
        let vec = vector_field.get_vector(thing.pos());
        thing.pos = thing.pos() + vec.normalize() * 5.0;
        thing.age += 1;
    }

    things.retain(|ea| bounds.contains(ea.pos()));

    for _ in (0..(MAX_THINGS - things.len())) {
        things.push(Thing::random(bounds));
    }
}

#[no_mangle]
pub fn view(app: &App, model: &Model, frame: Frame) {
    if !model.state.needs_draw {
        return;
    }

    let bounds = app.window_rect();
    let draw = app.draw();

    draw.background().color(BLACK);
    // draw.rect()
    //     .xy(bounds.xy())
    //     .wh(bounds.wh())
    //     .color(rgba(0.0, 0.0, 0.0, 0.01));

    let State {
        vector_field: vf,
        things,
        updates,
        ..
    } = &*model.state;

    if *updates == 1 {
        draw.background().color(BLACK);
    }

    // vf.draw(&draw);

    for thing in things {
        // let width = thing.age.min(20) as f32 * 0.1 * thing.width;
        draw.ellipse()
            .xy(thing.pos())
            .color(WHITE)
            // .stroke_color(BLACK)
            // .stroke_weight(1.0)
            .radius(thing.width);
        // let n = thing.positions.len();
        // for (i, (a, b)) in thing.positions.iter().tuple_windows().enumerate() {
        //     // draw.ellipse().xy(*b).color(WHITE).radius(5.0);
        //     let rel = i as f32 / n as f32;
        //     draw.line()
        //         .start(*a)
        //         .end(*b)
        //         // .color(rgba(rel, rel, rel, 1.0))
        //         .color(rgba(rel, 0.0, 0.0, 1.0))
        //         .caps_round()
        //         .stroke_weight(thing.width * rel);
        // }
    }

    draw.to_frame(app, &frame).unwrap();
}

fn gray(opacity: f32) -> Rgba {
    rgba(0.5, 0.5, 0.5, opacity)
}
