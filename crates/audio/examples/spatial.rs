use std::{io::Cursor, sync::Arc};

use ambient_audio::{
    blt::Lpf,
    hrtf::HrtfLib,
    track::Track,
    utils::{div_ceil, AvgIter},
    AudioEmitter, AudioListener, AudioStream, DynamicMix, Frame, SineWave, Source,
};
use circular_queue::CircularQueue;
use glam::{vec2, vec3, Mat3, Mat4, Vec3, Vec3Swizzles};
use itertools::izip;
use lyon::{
    lyon_tessellation::{BuffersBuilder, StrokeOptions, StrokeTessellator, VertexBuffers},
    math::point,
};
use macroquad::{
    color::colors,
    hash,
    models::Vertex,
    prelude::{
        draw_mesh, is_key_pressed, is_mouse_button_down, mouse_position, KeyCode, Mesh, BLACK,
        BLUE, GREEN, RED,
    },
    shapes::{draw_circle, draw_circle_lines, draw_line},
    ui::{root_ui, widgets::Window},
    window::{clear_background, next_frame, screen_height, screen_width},
};
use ordered_float::OrderedFloat;
use parking_lot::Mutex;

struct Plotter {
    tess: StrokeTessellator,
    geom: [VertexBuffers<Vertex, u16>; 2],
    mesh: [macroquad::models::Mesh; 2],
}

impl Plotter {
    fn new() -> Self {
        Self {
            tess: Default::default(),
            geom: [VertexBuffers::new(), VertexBuffers::new()],
            mesh: [
                Mesh {
                    vertices: Vec::new(),
                    indices: Vec::new(),
                    texture: None,
                },
                Mesh {
                    vertices: Vec::new(),
                    indices: Vec::new(),
                    texture: None,
                },
            ],
        }
    }

    fn build_graph(&mut self, points: impl Iterator<Item = Frame>, transform: Mat3) {
        let mut left = lyon::path::Path::builder();
        let mut right = lyon::path::Path::builder();

        let mut points = points.enumerate().map(|(x, s)| {
            let l = transform.transform_point2(vec2(x as f32, s.x));
            let r = transform.transform_point2(vec2(x as f32, s.y));

            (l, r)
        });

        let (l, r) = points.next().unwrap_or_default();

        left.begin(point(l.x, l.y));
        right.begin(point(r.x, r.y));

        for (l, r) in points {
            left.line_to(point(l.x, l.y));
            right.line_to(point(r.x, r.y));
        }

        left.end(false);
        right.end(false);

        let left = left.build();
        let right = right.build();

        let colors = [RED, BLUE];
        let paths = [&left, &right];

        for (geom, mesh, path, color) in izip!(&mut self.geom, &mut self.mesh, paths, colors) {
            geom.vertices.clear();
            geom.indices.clear();

            self.tess
                .tessellate_path(
                    path,
                    &StrokeOptions::default().with_line_width(2.0),
                    &mut BuffersBuilder::new(
                        geom,
                        |vertex: lyon::lyon_tessellation::StrokeVertex| {
                            let pos = vertex.position();
                            macroquad::models::Vertex {
                                position: macroquad::math::vec3(pos.x, pos.y, 0.0),
                                uv: Default::default(),
                                color,
                            }
                        },
                    ),
                )
                .unwrap();

            mesh.vertices.clear();
            mesh.vertices.extend_from_slice(&geom.vertices);

            mesh.indices.clear();
            mesh.indices.extend_from_slice(&geom.indices);
        }
    }

    fn draw(&self) {
        draw_mesh(&self.mesh[0]);
        draw_mesh(&self.mesh[1]);
    }
}

impl Default for Plotter {
    fn default() -> Self {
        Self::new()
    }
}

#[macroquad::main("Spatial Audio")]
async fn main() {
    let stream = AudioStream::new().unwrap();
    // let sink = stream.mixer().new_sink(5);
    // let history_len = stream.mixer().sample_rate() as usize / 440;
    let history_len = 512;

    let sample_history = Arc::new(Mutex::new(CircularQueue::with_capacity(history_len)));
    let sample_history2 = Arc::new(Mutex::new(Vec::new()));

    let hrtf_lib = HrtfLib::load(Cursor::new(include_bytes!(
        "../../world_audio/IRC_1002_C.bin"
    )))
    .unwrap();

    let listener = Arc::new(Mutex::new(AudioListener {
        transform: Mat4::IDENTITY,
        ear_distance: Vec3::X * 0.2,
    }));

    let spatial = Arc::new(Mutex::new(AudioEmitter {
        pos: Vec3::Z,
        attenuation: ambient_audio::Attenuation::InversePoly {
            quad: 0.0,
            lin: 0.0,
            constant: 1.0,
        },
        ..Default::default()
    }));

    let lpf = Arc::new(Mutex::new(Lpf {
        freq: 10000.0,
        bandwidth: 6.0,
    }));

    let ambience = Track::from_wav(
        std::fs::read("example_assets/ambience.wav")
            .unwrap()
            .to_vec(),
    )
    .unwrap()
    .decode()
    .repeat();

    let chord_source = SineWave::new(523.25);
    // .gain(0.25)
    // .mix(SineWave::new(659.25).gain(0.25))
    // .mix(SineWave::new(783.99).gain(0.25));

    let weights = Arc::new(Mutex::new(Box::from([0.0, 1.0])));

    let source = DynamicMix::new(
        vec![Box::new(ambience), Box::new(chord_source)],
        weights.clone(),
    )
    .blt(lpf.clone())
    .spatial(&hrtf_lib, listener.clone(), spatial.clone())
    .history(128.0, sample_history.clone())
    .oscilloscope(8, sample_history2.clone());

    stream.mixer().play(source);

    let emitters = vec![spatial];
    let mut listener_pos = Vec3::ZERO;

    let scale = vec2(64.0, -64.0);
    let dot_size = 0.1;
    let ear_size = 0.05;

    let mut plot = Plotter::new();
    let mut plot2 = Plotter::new();

    loop {
        let screen_size = vec2(screen_width(), screen_height());
        clear_background(colors::GRAY);
        let origin = screen_size / 2.0;

        // Update listener
        let (left, right) = {
            let mut l = listener.lock();
            l.transform = Mat4::from_translation(listener_pos);
            l.ear_positions()
        };

        draw_circle(
            origin.x + left.x * scale.x,
            origin.y + left.z * scale.y,
            ear_size * scale.x,
            RED,
        );

        draw_circle(
            origin.x + right.x * scale.x,
            origin.y + right.z * scale.y,
            ear_size * scale.x,
            BLUE,
        );

        draw_circle_lines(
            origin.x + listener_pos.x * scale.x,
            origin.y + listener_pos.z * scale.y,
            scale.x,
            1.0,
            BLUE,
        );

        for emitter in &emitters {
            let emitter = emitter.lock();
            draw_line(
                origin.x + left.x * scale.x,
                origin.y + left.z * scale.y,
                origin.x + emitter.pos.x * scale.x,
                origin.y + emitter.pos.z * scale.y,
                1.0,
                BLACK,
            );

            draw_line(
                origin.x + right.x * scale.x,
                origin.y + right.z * scale.y,
                origin.x + emitter.pos.x * scale.x,
                origin.y + emitter.pos.z * scale.y,
                1.0,
                BLACK,
            );

            draw_circle(
                origin.x + emitter.pos.x * scale.x,
                origin.y + emitter.pos.z * scale.y,
                dot_size * scale.x,
                GREEN,
            );
        }

        if is_mouse_button_down(macroquad::prelude::MouseButton::Left) {
            let mouse_pos = mouse_position();
            let mouse_pos =
                vec3(mouse_pos.0 - origin.x, 0.0, mouse_pos.1 - origin.y) / scale.extend(1.0).xzy();

            if let Some(emitter) = emitters
                .iter()
                .min_by_key(|v| OrderedFloat(v.lock().pos.distance(mouse_pos)))
            {
                let mut emitter = emitter.lock();
                if emitter.pos.distance(mouse_pos) < 2.0 {
                    emitter.pos = mouse_pos;
                }
            }
        }

        let mov = vec3(
            is_key_pressed(KeyCode::Right) as i32 as f32
                - is_key_pressed(KeyCode::Left) as i32 as f32,
            is_key_pressed(KeyCode::Up) as i32 as f32 - is_key_pressed(KeyCode::Down) as i32 as f32,
            0.0,
        ) * 0.25;

        listener_pos += mov;

        Window::new(
            hash!(),
            macroquad::math::vec2(0.0, screen_height() - 200.0),
            macroquad::math::vec2(screen_width(), 200.0),
        )
        .label("Controls")
        .ui(&mut root_ui(), |ui| {
            let mut weights = weights.lock();
            let mut lpf = lpf.lock();

            ui.slider(hash!(), "ambience", 0.0..1.0, &mut weights[0]);
            ui.slider(hash!(), "chord", 0.0..1.0, &mut weights[1]);
            ui.label(None, &format!("listener: {listener_pos}"));
            ui.slider(hash!(), "low pass", 0.0..10000.0, &mut lpf.freq);
            // ui.slider(hash!(), "bandwidth", 0.0..5.0, &mut bpf.bandwidth);
        });

        let width = screen_size.x;
        let height = 64.0;

        plot.build_graph(
            sample_history.lock().asc_iter().copied(),
            Mat3::from_translation(vec2(0.0, height))
                * Mat3::from_scale(vec2(width / history_len as f32, -height)),
        );

        {
            let sample_history2 = sample_history2.lock();
            let stride = div_ceil(sample_history2.len(), history_len);

            let samples = AvgIter::new(sample_history2.iter().copied(), stride);
            let count = samples.len();

            plot2.build_graph(
                samples,
                Mat3::from_translation(vec2(0.0, 2.0 * height))
                    * Mat3::from_scale(vec2((width / count as f32).ceil(), -height)),
            );
        }

        plot.draw();
        plot2.draw();

        next_frame().await;
    }
}
