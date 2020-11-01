use prometheus_exporter_base::prelude::*;
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct Statistics {
    pub accesses: HashMap<String, u64>,
}

impl Statistics {
    pub(crate) fn inc_page(&mut self, page: &str) {
        let original_value = *self.accesses.get(page).unwrap_or(&0);
        self.accesses.insert(page.to_owned(), original_value + 1);
    }

    pub(crate) fn render_to_prometheus(&self) -> String {
        let mut pc = PrometheusMetric::build()
            .with_name("nas_gallery_access_to_static_content")
            .with_metric_type(MetricType::Counter)
            .with_help("Access to static content")
            .build();

        self.accesses.iter().for_each(|(key, val)| {
            pc.render_and_append_instance(
                &PrometheusInstance::new()
                    .with_label("path", key.as_ref())
                    .with_value(*val),
            );
        });
        pc.render()
    }
}
