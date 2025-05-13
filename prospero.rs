//! wgpu prospero challenge

mod common;

use log::{LevelFilter, debug, info};
use regex::{Captures, Regex};

use std::fs::File;
use std::fs::read_to_string;
use std::io::BufWriter;
use std::path::PathBuf;
use std::time::SystemTime;
use std::u64;

fn generate_wgsl_distance_function(path: PathBuf) -> String {
    let re_comment = Regex::new(r"^#.*$").unwrap();
    let re_constant =
        Regex::new(r"^_([0-9a-f]+) +const +([+-]?[0-9]+\.?[0-9]*(e[+-]?[0-9]+)?)$").unwrap();
    let re_nullary = Regex::new(r"^_([0-9a-f]+) +(var-x|var-y)$").unwrap();
    let re_unary = Regex::new(r"^_([0-9a-f]+) +(neg|square|sqrt) +_([0-9a-f]+)$").unwrap();
    let re_binary =
        Regex::new(r"^_([0-9a-f]+) +(mul|add|sub|min|max) +_([0-9a-f]+) +_([0-9a-f]+)$").unwrap();

    info!("loading {:?}", path);

    let parse_hex = |caps: &Captures<'_>, index: usize| -> u64 {
        u64::from_str_radix(caps.get(index).unwrap().as_str(), 16).unwrap()
    };
    let parse_str =
        |caps: &Captures<'_>, index: usize| -> String { caps.get(index).unwrap().as_str().into() };

    let mut constants = vec![];

    let mut commands = vec![];
    let mut last_command_id = None;
    let mut push_command = |id: u64, value: String| {
        commands.push(format!("  let item_{:0x}: f32 = {};", id, value));
        last_command_id = Some(id);
    };

    for line in read_to_string(path).unwrap().lines() {
        if let Some(_) = re_comment.captures(line) {
            continue;
        }

        if let Some(caps) = re_constant.captures(line) {
            let id = parse_hex(&caps, 1);
            let value = parse_str(&caps, 2);
            debug!("constant {id} {value} \"{line}\"");
            constants.push(format!("const item_{:0x}: f32 = {};", id, value));
            continue;
        }

        if let Some(caps) = re_nullary.captures(line) {
            let id = parse_hex(&caps, 1);
            let command = parse_str(&caps, 2);
            debug!("nullary {id} {command}");
            let value = match command.as_str() {
                "var-x" => "pos.x",
                "var-y" => "pos.y",
                _ => unreachable!(),
            };
            push_command(id, value.into());
            continue;
        }

        if let Some(caps) = re_unary.captures(line) {
            let id = parse_hex(&caps, 1);
            let command = parse_str(&caps, 2);
            let value = parse_hex(&caps, 3);
            debug!("unary {id} {command} {value}");
            let value = match command.as_str() {
                "neg" => format!("-item_{:0x}", value),
                "square" => format!("item_{:0x} * item_{:0x}", value, value),
                "sqrt" => format!("sqrt(item_{:0x})", value),
                _ => unreachable!(),
            };
            push_command(id, value);
            continue;
        }

        if let Some(caps) = re_binary.captures(line) {
            let id = parse_hex(&caps, 1);
            let command = parse_str(&caps, 2);
            let value_left = parse_hex(&caps, 3);
            let value_right = parse_hex(&caps, 4);
            debug!("binary {id} {command} {value_left} {value_right}");
            let value = match command.as_str() {
                "mul" => format!("item_{:0x} * item_{:0x}", value_left, value_right),
                "add" => format!("item_{:0x} + item_{:0x}", value_left, value_right),
                "sub" => format!("item_{:0x} - item_{:0x}", value_left, value_right),
                "min" => format!("min(item_{:0x}, item_{:0x})", value_left, value_right),
                "max" => format!("max(item_{:0x}, item_{:0x})", value_left, value_right),
                _ => unreachable!(),
            };
            push_command(id, value);
            continue;
        }

        panic!("unhandled \"{}\"", line);
    }

    let constants = constants.join("\n");
    let commands = format!(
        "fn signed_distance_function(pos: vec3<f32>) -> f32 {{\n{}\n  return item_{:0x};\n}}",
        commands.join("\n"),
        last_command_id.unwrap(),
    );

    format!("{}\n\n{}", constants, commands)
}

fn save_grayscale_image(path: PathBuf, colors: &Vec<u8>, image_size: usize) {
    info!("saving {:?}", path);

    let file = File::create(path).unwrap();
    let handle = BufWriter::new(file);
    let mut encoder = png::Encoder::new(handle, image_size as u32, image_size as u32);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));
    let source_chromaticities = png::SourceChromaticities::new(
        (0.31270, 0.32900),
        (0.64000, 0.33000),
        (0.30000, 0.60000),
        (0.15000, 0.06000),
    );
    encoder.set_source_chromaticities(source_chromaticities);
    let mut writer = encoder.write_header().unwrap();

    writer.write_image_data(&colors).unwrap();
}

#[derive(Debug)]
struct SecondDurations {
    init: f32,
    generate: f32,
    compile: f32,
    alloc: f32,
    eval: f32,
    retrieve: f32,
    export: f32,
    total: f32,
}

impl SecondDurations {
    fn from_tops(
        top_aa: SystemTime,
        top_bb: SystemTime,
        top_cc: SystemTime,
        top_dd: SystemTime,
        top_ee: SystemTime,
        top_ff: SystemTime,
        top_gg: SystemTime,
        top_hh: SystemTime,
    ) -> Self {
        Self {
            init: top_bb.duration_since(top_aa).unwrap().as_secs_f32(),
            generate: top_cc.duration_since(top_bb).unwrap().as_secs_f32(),
            compile: top_dd.duration_since(top_cc).unwrap().as_secs_f32(),
            alloc: top_ee.duration_since(top_dd).unwrap().as_secs_f32(),
            eval: top_ff.duration_since(top_ee).unwrap().as_secs_f32(),
            retrieve: top_gg.duration_since(top_ff).unwrap().as_secs_f32(),
            export: top_hh.duration_since(top_gg).unwrap().as_secs_f32(),
            total: top_hh.duration_since(top_aa).unwrap().as_secs_f32(),
        }
    }
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Info)
        .format_timestamp_micros()
        .init();

    let image_size: usize = 1024;
    info!("image_size {}", image_size);

    let top_aa = SystemTime::now();

    let common = common::Common::default();

    let top_bb = SystemTime::now();

    let source = generate_wgsl_distance_function("prospero.vm".into());

    info!("compiling shader");
    let top_cc = SystemTime::now();

    let sample_plane = common::SamplePlane::from_common_with_sdf_source(&common, &source);

    info!("allocating distances");
    let top_dd = SystemTime::now();

    let distance_buffer = common.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("distances"),
        size: (image_size * image_size * 4) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    info!("evaluating distances");
    let top_ee = SystemTime::now();

    sample_plane.run(&distance_buffer, image_size);

    info!("retrieving distances");
    let top_ff = SystemTime::now();

    let distances = common.retrieve_buffer::<f32>(&distance_buffer);
    assert!(distances.len() == image_size * image_size);

    info!("preparing colors");
    let top_gg = SystemTime::now();

    let colors = distances
        .into_iter()
        .map(|vv| if vv < 0.0 { 255 } else { 0 })
        .collect::<Vec<u8>>();
    save_grayscale_image("prospero.png".into(), &colors, image_size);

    let top_hh = SystemTime::now();

    let timings = SecondDurations::from_tops(
        top_aa, top_bb, top_cc, top_dd, top_ee, top_ff, top_gg, top_hh,
    );
    info!("timings {:#?}", timings);
}
