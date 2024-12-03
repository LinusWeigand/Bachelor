use charming::datatype::{CompositeValue, NumericValue};
use charming::element::{AreaStyle, Symbol, Trigger};
use charming::element::{AxisType, Color, ItemStyle};
use charming::series::Line;
use charming::{component::Axis, element::Tooltip, Chart};

use super::server_etl::MetricData;
use super::DURATION;

fn calculate_moving_average(data: &[(f64, f64)], window_size: usize) -> Vec<(f64, f64)> {
    let mut moving_average = Vec::new();
    let mut sum = 0.0;

    for (i, &(_, y)) in data.iter().enumerate() {
        sum += y;
        if i >= window_size {
            sum -= data[i - window_size].1;
            moving_average.push((data[i].0, sum / window_size as f64));
        } else {
            moving_average.push((data[i].0, sum / (i + 1) as f64));
        }
    }

    moving_average
}

pub fn get_cpu_chart(cpu_data: &Vec<Vec<(f64, f64)>>) -> String {
    let data_set: Vec<Vec<CompositeValue>> = cpu_data
        .iter()
        .map(|inner_vec| {
            inner_vec
                .iter()
                .map(|(x, y)| {
                    CompositeValue::Array(vec![
                        CompositeValue::Number(NumericValue::from(*x)),
                        CompositeValue::Number(NumericValue::from(*y)),
                    ])
                })
                .collect()
        })
        .collect();

    let mut chart = Chart::new()
        .x_axis(
            Axis::new()
                .type_(AxisType::Value)
                .boundary_gap(false)
                .min(0)
                .max(DURATION),
        )
        .y_axis(Axis::new().type_(AxisType::Value).boundary_gap(false))
        .tooltip(Tooltip::new().trigger(Trigger::Axis));

    for (i, data) in data_set.iter().enumerate() {
        chart = chart.series(
            Line::new()
                .name(&format!("Core {}", i))
                .data(data.clone())
                .symbol(Symbol::None),
        );
    }
    let options = serde_json::to_string(&chart).unwrap_or_default();
    options
}

pub fn get_chart(
    data: &Vec<(f64, f64)>,
    name: &str,
    max_y: Option<f32>,
    moving_average: bool,
) -> String {
    let data_set: Vec<CompositeValue> = data
        .iter()
        .map(|(x, y)| {
            CompositeValue::Array(vec![
                CompositeValue::Number(NumericValue::from(*x)),
                CompositeValue::Number(NumericValue::from(*y)),
            ])
        })
        .collect();

    let mut y_axis = Axis::new()
        .type_(AxisType::Value)
        .min(0)
        .boundary_gap(false);

    if let Some(max_y) = max_y {
        y_axis = y_axis.max(max_y);
    }

    let mut chart = Chart::new()
        .x_axis(
            Axis::new()
                .type_(AxisType::Value)
                .boundary_gap(false)
                .min(0)
                .max(DURATION),
        )
        .y_axis(y_axis)
        .tooltip(Tooltip::new().trigger(Trigger::Axis))
        .series(
            Line::new()
                .name(name)
                .data(data_set)
                .symbol(Symbol::None)
                .area_style(AreaStyle::new()),
        );

    let mut moving_average_set: Vec<CompositeValue> = Vec::new();
    if moving_average {
        moving_average_set = calculate_moving_average(data, 200)
            .iter()
            .map(|(x, y)| {
                CompositeValue::Array(vec![
                    CompositeValue::Number(NumericValue::from(*x)),
                    CompositeValue::Number(NumericValue::from(*y)),
                ])
            })
            .collect();
        chart = chart.series(
            Line::new()
                .name("Moving Average")
                .data(moving_average_set)
                .symbol(Symbol::None)
                .item_style(ItemStyle::new().color("#FF0000"))
                .smooth(true),
        )
    }

    serde_json::to_string(&chart).unwrap()
}

pub struct MetricDataJson {
    pub cpu: String,
    pub ram: String,
    pub send: String,
    pub received: String,
    pub net_io: String,
    pub send_io: String,
    pub receive_io: String,
    pub disk_io: String,
}

pub fn get_instance_metric_data(server_version: u8, mode: &str) -> MetricDataJson {
    let data = MetricData::new(&format!("v{}/{}", server_version, mode));

    let cpu = get_cpu_chart(&data.cpu_util);
    let ram = get_chart(&data.ram, "GB", None, false);
    let send = get_chart(&data.send, "GB/s", Some(3.1), true);
    let received = get_chart(&data.received, "GB/s", Some(0.01), true);
    let disk_io = get_chart(&data.disk_io, "Percentage", None, false);
    let net_io = get_chart(&data.net_io, "Percentage", None, false);
    let send_io = get_chart(&data.send_io, "Percentage", None, false);
    let receive_io = get_chart(&data.receive_io, "Percentage", None, false);
    // println!("{}", disk_io);
    MetricDataJson {
        cpu,
        ram,
        send,
        received,
        net_io,
        send_io,
        receive_io,
        disk_io,
    }
}
