//! wgpu compute benchmark

mod common;
mod options;

use log::{LevelFilter, debug, info};

use wgpu::util::DeviceExt;

use plotters::prelude::*;

use rand::distr::StandardUniform;
use rand::prelude::*;

use std::collections::BTreeMap;
use std::time::SystemTime;

use plotters::style::full_palette::DEEPORANGE;
use plotters::style::full_palette::DEEPPURPLE;
use plotters::style::full_palette::GREEN;
use plotters::style::full_palette::LIGHTGREEN;
use plotters::style::full_palette::ORANGE;
use plotters::style::full_palette::RED;
use plotters::style::full_palette::YELLOW;

fn u64_norm(aa: u64, bb: u64) -> u64 {
    if aa > bb { aa - bb } else { bb - aa }
}

fn test_affine(common: &common::Common, inputs: &Vec<u64>, check_results: bool) -> f32 {
    let affine = common::Affine::from_common(&common);

    let input_buffer = common
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("input"),
            contents: bytemuck::cast_slice(&inputs),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });
    assert!(input_buffer.size() % 8 == 0);
    let output_buffer = common.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("output"),
        size: input_buffer.size(),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    assert!(output_buffer.size() % 8 == 0);

    let top_start = SystemTime::now();
    affine.run(&input_buffer, &output_buffer, inputs.len());
    let top_finish = SystemTime::now();

    let duration = top_finish.duration_since(top_start).unwrap();
    let duration = duration.as_secs_f32();

    let results = common.retrieve_buffer::<u64>(&output_buffer);
    debug!("affine_results {:?}", results);

    if check_results {
        assert!(results.len() == inputs.len());
        let mut max_error = 0;
        for (rr, ii) in results.iter().zip(inputs) {
            let rr_ = 100 + 2 * ii;
            max_error = u64::max(max_error, u64_norm(*rr, rr_));
        }
        assert!(max_error == 0);
    }

    duration
}

fn test_sample_sdf_sphere(common: &common::Common, inputs: &Vec<f32>, check_results: bool) -> f32 {
    let sample_sdf = common::SampleDistance::from_common_with_sdf(&common, "sphere");
    let (inputs, results, duration) = test_sample_sdf(common, &sample_sdf, inputs);

    if check_results {
        assert!(results.len() * 3 == inputs.len());
        let mut max_error = 0.0;
        for (kk, rr) in results.iter().enumerate() {
            let px = inputs[3 * kk + 0];
            let py = inputs[3 * kk + 1];
            let pz = inputs[3 * kk + 2];
            let rr_ = f32::sqrt(px * px + py * py + pz * pz) - 0.8;
            max_error = f32::max(max_error, f32::abs(rr_ - rr));
        }
        assert!(max_error < 1e-5);
    }

    duration
}

fn make_test_sample_sdf_from_name(
    common: &common::Common,
    sdf_name: &str,
) -> impl Fn(&Vec<f32>) -> f32 {
    let sample_sdf = common::SampleDistance::from_common_with_sdf(&common, sdf_name);
    move |inputs: &Vec<f32>| test_sample_sdf(&common, &sample_sdf, inputs).2
}

fn test_sample_sdf(
    common: &common::Common,
    sample_sdf: &common::SampleDistance,
    inputs: &Vec<f32>,
) -> (Vec<f32>, Vec<f32>, f32) {
    let mut inputs = inputs.clone();
    while !inputs.is_empty() && inputs.len() % 3 != 0 {
        inputs.pop();
    }
    assert!(inputs.len() % 3 == 0);
    let position_buffer = common
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("position"),
            contents: bytemuck::cast_slice(&inputs),
            usage: wgpu::BufferUsages::STORAGE,
        });
    assert!(position_buffer.size() % (3 * 4) == 0);
    let distance_buffer = common.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("distance"),
        size: position_buffer.size() / 3,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    assert!(distance_buffer.size() % 4 == 0);

    let top_start = SystemTime::now();
    assert!(inputs.len() % 3 == 0);
    sample_sdf.run(&position_buffer, &distance_buffer, inputs.len() / 3);
    let top_finish = SystemTime::now();

    let duration = top_finish.duration_since(top_start).unwrap();
    let duration = duration.as_secs_f32();

    let results = common.retrieve_buffer::<f32>(&distance_buffer);
    debug!("sample_sdf_results {:?}", results);

    (inputs, results, duration)
}

fn make_test_sample_plane_from_name(
    common: &common::Common,
    sdf_name: &str,
) -> impl Fn(&Vec<f32>) -> f32 {
    let sdf_path = format!("shaders/sdf/{}.wgsl", sdf_name);
    let sdf_source = std::fs::read_to_string(sdf_path).expect("Can't load sdf");
    let sample_plane =
        common::SamplePlane::from_common_with_sdf_source(&common, sdf_source.as_str());
    move |inputs: &Vec<f32>| test_sample_plane(&common, &sample_plane, inputs).1
}

fn test_sample_plane(
    common: &common::Common,
    sample_plane: &common::SamplePlane,
    inputs: &Vec<f32>,
) -> (Vec<f32>, f32) {
    let mut image_size = inputs.len().isqrt();
    image_size -= image_size % 16;
    assert!(image_size > 0);
    assert!(image_size % 16 == 0);

    let distance_buffer = common.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("distance"),
        size: (image_size * image_size * 4) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    assert!(distance_buffer.size() % (16 * 16 * 4) == 0);

    let top_start = SystemTime::now();
    assert!(image_size % 16 == 0);
    sample_plane.run(&distance_buffer, image_size);
    let top_finish = SystemTime::now();

    let duration = top_finish.duration_since(top_start).unwrap();
    let duration = duration.as_secs_f32();

    let results = common.retrieve_buffer::<f32>(&distance_buffer);
    debug!("sample_plane_results {:?}", results);

    (results, duration)
}

enum Callback<'a> {
    FloatL(&'a dyn Fn(&Vec<f32>) -> f32),
    Float(fn(&common::Common, &Vec<f32>, bool) -> f32),
    Usize(fn(&common::Common, &Vec<u64>, bool) -> f32),
}

type NamedSeries<'a> = BTreeMap<String, (RGBColor, Vec<(f32, f32)>, Callback<'a>)>;

fn plot_named_series(common: &common::Common, named_series: &NamedSeries, name: &str) {
    let mut max_duration = 0.0f32;
    let mut max_size = 0.0f32;
    for (_, series, _) in named_series.values() {
        let max_duration_ = series.iter().map(|xx| xx.1).reduce(f32::max).unwrap();
        let max_size_ = series.iter().map(|xx| xx.0).reduce(f32::max).unwrap();
        max_duration = f32::max(max_duration, max_duration_);
        max_size = f32::max(max_size, max_size_);
    }
    let max_duration = f32::ceil(max_duration);
    let max_size = f32::ceil(max_size) + 8.0;

    let path = format!("figures/{name}.png");
    info!("saving \"{path}\" {max_duration:1.0}ms");
    let root = BitMapBackend::new(&path, (640, 480)).into_drawing_area();

    root.fill(&WHITE).unwrap();

    let adapter_info = common.adapter.get_info();
    let caption = format!(
        "duration [ms] vs. input size {} {}",
        adapter_info.name, adapter_info.driver_info,
    );
    let mut chart = ChartBuilder::on(&root)
        .caption(caption, ("sans-serif", 22).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0f32..max_size, 0f32..max_duration)
        .unwrap();

    chart.configure_mesh().draw().unwrap();

    for (name, (color, series, _)) in named_series {
        chart
            .draw_series(LineSeries::new(series.clone(), color.clone()))
            .unwrap()
            .label(name)
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color.clone()));
    }

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()
        .unwrap();

    root.present().unwrap();
}

fn main() {
    let options = options::parse_options();

    // To change the log level, set the `RUST_LOG` environment variable.
    let log_filter = match options.skip_long_tests {
        true => LevelFilter::Debug,
        false => LevelFilter::Info,
    };
    env_logger::Builder::from_default_env()
        .filter_level(log_filter)
        .format_timestamp_micros()
        .init();

    let common = common::Common::default();

    {
        let inputs: Vec<u64> = vec![10, 11, 12, 13];
        test_affine(&common, &inputs, true);
    }

    let test_sample_sdf_alien = make_test_sample_sdf_from_name(&common, "alien");
    let test_sample_sdf_cheese = make_test_sample_sdf_from_name(&common, "cheese");
    let test_sample_sdf_arlo = make_test_sample_sdf_from_name(&common, "arlo");

    {
        #[rustfmt::skip]
        let inputs = vec![
            0.0, 0.0, 0.0,
            1.0, 0.0, 0.0,
            0.0, 1.0, 0.0,
            0.0, 0.0, 1.0,
            1.0, 2.0, 3.0,
        ];
        test_sample_sdf_sphere(&common, &inputs, true);
        test_sample_sdf_alien(&inputs);
        test_sample_sdf_cheese(&inputs);
        test_sample_sdf_arlo(&inputs);
    }

    let test_sample_plane_alien = make_test_sample_plane_from_name(&common, "alien");
    let test_sample_plane_cheese = make_test_sample_plane_from_name(&common, "cheese");
    let test_sample_plane_arlo = make_test_sample_plane_from_name(&common, "arlo");

    if options.skip_long_tests {
        return;
    }

    {
        #[rustfmt::skip]
        let mut inputs = vec![];
        for _ in 0..32 {
            for _ in 0..32 {
                inputs.push(0.0);
            }
        }
        test_sample_plane_alien(&inputs);
        test_sample_plane_cheese(&inputs);
        test_sample_plane_arlo(&inputs);
    }

    let mut named_series = NamedSeries::new();

    named_series.insert("affine".into(), (RED, vec![], Callback::Usize(test_affine)));
    named_series.insert(
        "sample_sphere".into(),
        (YELLOW, vec![], Callback::Float(test_sample_sdf_sphere)),
    );
    named_series.insert(
        "sample_alien".into(),
        (LIGHTGREEN, vec![], Callback::FloatL(&test_sample_sdf_alien)),
    );
    named_series.insert(
        "sample_cheese".into(),
        (ORANGE, vec![], Callback::FloatL(&test_sample_sdf_cheese)),
    );
    named_series.insert(
        "sample_arlo".into(),
        (RED, vec![], Callback::FloatL(&test_sample_sdf_arlo)),
    );
    named_series.insert(
        "plane_alien".into(),
        (GREEN, vec![], Callback::FloatL(&test_sample_plane_alien)),
    );
    named_series.insert(
        "plane_cheese".into(),
        (
            DEEPORANGE,
            vec![],
            Callback::FloatL(&test_sample_plane_cheese),
        ),
    );
    named_series.insert(
        "plane_arlo".into(),
        (
            DEEPPURPLE,
            vec![],
            Callback::FloatL(&test_sample_plane_arlo),
        ),
    );

    {
        let label = named_series
            .keys()
            .map(String::clone)
            .collect::<Vec<String>>()
            .join(" ");
        info!("nn {}", label);
    }

    let mut rng = rand::rng();
    for ll in (options.start_range..=options.finish_range).step_by(options.step_range as usize) {
        let mut inputs_usize: Vec<u64> = vec![];
        let mut inputs_float: Vec<f32> = vec![];
        for _ in 0..(ll * 1024) {
            inputs_usize.push(rng.random_range(0u64..100u64));
            inputs_float.push(rng.sample(StandardUniform));
        }

        let mut stats = vec![];
        for (_, series, callback) in named_series.values_mut() {
            let mut total_duration = 0.0;
            let mut count = 0;
            while total_duration < options.max_spin_duration {
                let check_results = count < 2;
                let duration = match callback {
                    Callback::Usize(callback) => callback(&common, &inputs_usize, check_results),
                    Callback::Float(callback) => callback(&common, &inputs_float, check_results),
                    Callback::FloatL(callback) => callback(&inputs_float),
                };
                let duration = 1e3 * duration;
                total_duration += duration;
                count += 1;
            }
            assert!(count != 0);
            let mean_duration = total_duration / count as f32;
            series.push((ll as f32, mean_duration));
            stats.push((count, mean_duration));
        }

        {
            let label = stats
                .iter()
                .map(|(cc, dd)| format!("x{cc:02} {dd:5.3}ms"))
                .collect::<Vec<String>>()
                .join(" ");
            info!("{}k {}", ll, label);
        }
    }

    {
        let label = named_series
            .keys()
            .map(String::clone)
            .collect::<Vec<String>>()
            .join(" ");
        info!("nn {}", label);
    }

    plot_named_series(&common, &named_series, "kernel_duration");
}
