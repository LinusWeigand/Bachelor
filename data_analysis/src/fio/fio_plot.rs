use charming::datatype::{CompositeValue, NumericValue};
use charming::element::AxisType;
use charming::element::{AreaStyle, ItemStyle, Symbol, Trigger};
use charming::series::Line;
use charming::{component::Axis, element::Tooltip, Chart};

use super::fio_etl::MetricData;

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

pub fn get_metric_chart(metric_data: Vec<(f64, f64)>) -> String {
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
    let moving_average_data = calculate_moving_average(&metric_data, 60);
    let moving_avg_set: Vec<CompositeValue> = moving_average_data
        .iter()
        .map(|&(x, y)| {
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
                .max(60000),
        )
        .y_axis(Axis::new().type_(AxisType::Value).boundary_gap(false))
        .tooltip(Tooltip::new().trigger(Trigger::Axis))
        .series(
            Line::new()
                .name("Original")
                .area_style(AreaStyle::new())
                .data(data_set)
                .symbol(Symbol::None),
        )
        .series(
            Line::new()
                .name("Moving Average")
                .data(moving_avg_set)
                .item_style(ItemStyle::new().color("#FF0000"))
                .smooth(true)
                .symbol(Symbol::None),
        );

    let options = serde_json::to_string(&chart).unwrap_or_default();
    options
}

pub enum TestType {
    SequentialRead,
    SequentialWrite,
    RealWorld,
}

pub struct MetricDataJson {
    pub bw: String,
    pub iops: String,
    pub lat: String,
    pub clat: String,
}

pub fn get_instance_metric_data(
    instance: &str,
    raid_level: u8,
    test_type: TestType,
) -> MetricDataJson {
    let test_type_str = match test_type {
        TestType::SequentialRead => "seq_read",
        TestType::SequentialWrite => "seq_write",
        TestType::RealWorld => "real_world",
    };
    let data = MetricData::new(&format!(
        "{}/RAID{}/{}",
        instance, raid_level, test_type_str
    ));
    let bw = get_metric_chart(data.bw);
    let iops = get_metric_chart(data.iops);
    let lat = get_metric_chart(data.lat);
    let clat = get_metric_chart(data.clat);
    MetricDataJson {
        bw,
        iops,
        lat,
        clat,
    }
}
