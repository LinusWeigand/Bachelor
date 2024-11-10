use charming::component::FilterMode;
use charming::{component::Axis, element::Tooltip, Chart};
use charming::component::DataZoom;
use charming::{element::AxisType, series::Bar};


pub fn get_avg_query_size_chart(warehouse_data: Vec<(u32, f64)>) -> String {
    let (names, values): (Vec<u32>, Vec<f64>) = warehouse_data.into_iter().map(|(s, f)| (s, f)).unzip();
    let names = names.into_iter().map(|id| format!("{}", id)).collect();
    let chart = Chart::new()
        .x_axis(
            Axis::new()
                .type_(AxisType::Category)
                .name("Warehouse ID")
                .data(names),

        )
        .y_axis(
            Axis::new()
                .type_(AxisType::Value)
                .name("Average GB per Query")
                .scale(true),
        )
        .tooltip(Tooltip::new())
        .data_zoom(
            DataZoom::new()
                .x_axis_index(0)
                .y_axis_index(0)
                .filter_mode(FilterMode::None)
                .start(0)
                .end(100),
        )
        .series(Bar::new().data(values));

    let options = serde_json::to_string(&chart).unwrap_or_default();
    options
}

pub fn get_buckets() -> String {
    let chart = Chart::new()
        .x_axis(
            Axis::new()
                .type_(AxisType::Category)
                .name("Percentile Group")
                .data((0..=100).map(|i| i.to_string()).collect::<Vec<_>>()),  // Percentile groups from 0 to 100
        )
        .y_axis(Axis::new().type_(AxisType::Value).name("Query Count"))
        .tooltip(Tooltip::new())
        .data_zoom(DataZoom::new().x_axis_index(0).y_axis_index(0))
        .series(Bar::new().data(vec![
            1355788, 31708, 698808, 684281, 688695, 729679, 701180, 672363, 663912, 698943, 
            684673, 691829, 699179, 684454, 692127, 691519, 695911, 690590, 689443, 691333, 
            692573, 694323, 688566, 691897, 693263, 692421, 691380, 690627, 691338, 692482, 
            694599, 688382, 692113, 691776, 691588, 692950, 693603, 689583, 691132, 691958, 
            692987, 692466, 689876, 692295, 691462, 691884, 692543, 691788, 691100, 691667, 
            692069, 691572, 691958, 691684, 692095, 691547, 691820, 692072, 691570, 691882, 
            691759, 691823, 691818, 691898, 691744, 691866, 691775, 691821, 691858, 691784, 
            691825, 691824, 691822, 691812, 691968, 691685, 691809, 691829, 691814, 691824, 
            691817, 691820, 691836, 691811, 691816, 691826, 691816, 691819, 691871, 691771, 
            691961, 691680, 691826, 691821, 691815, 691823, 691818, 691823, 691819, 691820, 
            1
        ]));

    let options = serde_json::to_string(&chart).unwrap_or_default();
    options
}
