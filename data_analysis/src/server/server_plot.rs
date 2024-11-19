use charming::datatype::{CompositeValue, NumericValue};
use charming::element::AxisType;
use charming::element::{AreaStyle, ItemStyle, Symbol, Trigger};
use charming::series::Line;
use charming::{component::Axis, element::Tooltip, Chart};

use super::server_etl::{MetricData};
use super::utils::{CPU, RAM};
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

pub fn get_cpu_chart(cpu_data: Vec<Vec<(f64, CPU)>>) -> String {
    let start_timestamp = cpu_data[0][0].0;
    let data_set: Vec<Vec<CompositeValue>> = cpu_data
        .clone()
        .into_iter()
        .map(|inner_vec| {
            inner_vec
                .into_iter()
                .map(|(x, y)| {
                    let active_time = y.user + y.system + y.nice + y.softirq + y.irq;
                    let total_time = active_time + y.idle + y.iowait;
                    let utilization = match total_time {
                        t if t <= 0. => 0.,
                        _ => active_time / total_time * 100.,
                    };
                    CompositeValue::Array(vec![
                        CompositeValue::Number(NumericValue::from(x - start_timestamp)),
                        CompositeValue::Number(NumericValue::from(utilization)),
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

pub fn get_ram_chart(ram_data: Vec<(f64, RAM)>) -> String {
    let start_timestamp = ram_data[0].0;
    let data_set: Vec<CompositeValue> = ram_data
        .clone()
        .into_iter()
        .map(|(x, y)| {
            CompositeValue::Array(vec![
                CompositeValue::Number(NumericValue::from(x - start_timestamp)),
                CompositeValue::Number(NumericValue::from(y.total - y.available)),
            ])
        })
        .collect();
    let chart = Chart::new()
        .x_axis(
            Axis::new()
                .type_(AxisType::Value)
                .boundary_gap(false)
                .min(0)
                .max(DURATION),
        )
        .y_axis(Axis::new().type_(AxisType::Value).boundary_gap(false))
        .tooltip(Tooltip::new().trigger(Trigger::Axis))
        .series(
            Line::new()
                .name("Original")
                .area_style(AreaStyle::new())
                .data(data_set)
                .symbol(Symbol::None),
        );
    let options = serde_json::to_string(&chart).unwrap_or_default();
    options
}

pub fn get_metric_chart(metric_data: Vec<(f64, f64)>, max_y: f32) -> String {
    let data_set: Vec<CompositeValue> = metric_data
        .clone()
        .into_iter()
        .map(|(x, y)| {
            CompositeValue::Array(vec![
                CompositeValue::Number(NumericValue::from(x)),
                CompositeValue::Number(NumericValue::from(y)),
            ])
        })
        .collect();
    let chart = Chart::new()
        .x_axis(
            Axis::new()
                .type_(AxisType::Value)
                .boundary_gap(false)
                .min(0)
                .max(DURATION),
        )
        .y_axis(
            Axis::new()
                .type_(AxisType::Value)
                .max(max_y)
                .min(0)
                .boundary_gap(false),
        )
        .tooltip(Tooltip::new().trigger(Trigger::Axis))
        .series(
            Line::new()
                .name("Original")
                .area_style(AreaStyle::new())
                .data(data_set)
                .symbol(Symbol::None),
        );
    let options = serde_json::to_string(&chart).unwrap_or_default();
    options
}

pub struct MetricDataJson {
    pub cpu: String,
    pub ram: String,
    pub send: String,
    pub received: String,
}

pub fn get_instance_metric_data(server_version: u8, mode: &str) -> MetricDataJson {
    let data = MetricData::new(&format!("v{}/{}", server_version, mode));

    let send = get_metric_chart(data.send, 4.);
    let received = get_metric_chart(data.received, 3000.);
    let cpu = get_cpu_chart(data.cpu);
    let ram = get_ram_chart(data.ram);
    MetricDataJson {
        cpu,
        ram,
        send,
        received,
    }
}
