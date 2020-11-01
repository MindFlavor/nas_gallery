use crate::options::Options;
use prometheus_exporter_base::prelude::*;
use rocket::State;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[inline]
pub(crate) fn track_authorized_static(
    options: &State<'_, Options>,
    statistics: &State<'_, Arc<RwLock<Statistics>>>,
    path: &str,
) {
    // only keep track of the accesses if the
    // prometheus exporting has been
    // enabled!
    if let Some(_) = options.prometheus_metrics_port {
        statistics.write().unwrap().inc_authorized_static(path);
    }
}

#[inline]
pub(crate) fn track_authorized_dynamic(
    options: &State<'_, Options>,
    statistics: &State<'_, Arc<RwLock<Statistics>>>,
) {
    // only keep track of the accesses if the
    // prometheus exporting has been
    // enabled!
    if let Some(_) = options.prometheus_metrics_port {
        statistics.write().unwrap().authorized_dynamic += 1;
    }
}

#[inline]
pub(crate) fn track_unauthorized(
    options: &State<'_, Options>,
    statistics: &State<'_, Arc<RwLock<Statistics>>>,
    path: &str,
) {
    // only keep track of the accesses if the
    // prometheus exporting has been
    // enabled!
    if let Some(_) = options.prometheus_metrics_port {
        statistics.write().unwrap().inc_unathorized(path);
    }
}

#[inline]
pub(crate) fn track_authorized_not_found(
    options: &State<'_, Options>,
    statistics: &State<'_, Arc<RwLock<Statistics>>>,
) {
    // only keep track of the accesses if the
    // prometheus exporting has been
    // enabled!
    if let Some(_) = options.prometheus_metrics_port {
        statistics.write().unwrap().authorized_not_found += 1;
    }
}

#[inline]
pub(crate) fn track_unauthorized_thumb(
    options: &State<'_, Options>,
    statistics: &State<'_, Arc<RwLock<Statistics>>>,
) {
    // only keep track of the accesses if the
    // prometheus exporting has been
    // enabled!
    if let Some(_) = options.prometheus_metrics_port {
        statistics.write().unwrap().unauthorized_thumb += 1;
    }
}

#[inline]
pub(crate) fn track_authorized_thumb(
    options: &State<'_, Options>,
    statistics: &State<'_, Arc<RwLock<Statistics>>>,
) {
    // only keep track of the accesses if the
    // prometheus exporting has been
    // enabled!
    if let Some(_) = options.prometheus_metrics_port {
        statistics.write().unwrap().authorized_thumb += 1;
    }
}

#[derive(Default, Debug)]
pub struct Statistics {
    pub authorized_static: HashMap<String, u64>,
    pub unathorized: HashMap<String, u64>,
    pub authorized_dynamic: u64,
    pub authorized_not_found: u64,
    pub authorized_thumb: u64,
    pub unauthorized_thumb: u64,
}

impl Statistics {
    pub(crate) fn inc_authorized_static(&mut self, page: &str) {
        if let Some(original_value) = self.authorized_static.get_mut(page) {
            *original_value += 1;
        } else {
            self.authorized_static.insert(page.to_owned(), 1);
        }
    }

    pub(crate) fn inc_unathorized(&mut self, page: &str) {
        if let Some(original_value) = self.unathorized.get_mut(page) {
            *original_value += 1;
        } else {
            self.unathorized.insert(page.to_owned(), 1);
        }
    }

    pub(crate) fn render_to_prometheus(&self) -> String {
        let mut s = String::new();

        let mut pc = PrometheusMetric::build()
            .with_name("nas_gallery_authorized_access_to_static_content")
            .with_metric_type(MetricType::Counter)
            .with_help("Authorized access to static content")
            .build();

        self.authorized_static.iter().for_each(|(key, val)| {
            pc.render_and_append_instance(
                &PrometheusInstance::new()
                    .with_label("path", key.as_ref())
                    .with_value(*val),
            );
        });
        s.push_str(&pc.render());

        let mut pc = PrometheusMetric::build()
            .with_name("nas_gallery_unauthorized_access")
            .with_metric_type(MetricType::Counter)
            .with_help("Unauthorized access")
            .build();

        self.unathorized.iter().for_each(|(key, val)| {
            pc.render_and_append_instance(
                &PrometheusInstance::new()
                    .with_label("path", key.as_ref())
                    .with_value(*val),
            );
        });
        s.push_str(&pc.render());

        s.push_str(
            &PrometheusMetric::build()
                .with_name("nas_gallery_authorized_access_to_dynamic_content")
                .with_metric_type(MetricType::Counter)
                .with_help("Authorized access to dynamic content")
                .build()
                .render_and_append_instance(
                    &PrometheusInstance::new().with_value(self.authorized_dynamic),
                )
                .render(),
        );

        s.push_str(
            &PrometheusMetric::build()
                .with_name("nas_gallery_authorized_not_found")
                .with_metric_type(MetricType::Counter)
                .with_help("Authorized access to not found content")
                .build()
                .render_and_append_instance(
                    &PrometheusInstance::new().with_value(self.authorized_not_found),
                )
                .render(),
        );

        s.push_str(
            &PrometheusMetric::build()
                .with_name("nas_gallery_authorized_thumb")
                .with_metric_type(MetricType::Counter)
                .with_help("Authorized access to thumbnail")
                .build()
                .render_and_append_instance(
                    &PrometheusInstance::new().with_value(self.authorized_thumb),
                )
                .render(),
        );

        s.push_str(
            &PrometheusMetric::build()
                .with_name("nas_gallery_unauthorized_thumb")
                .with_metric_type(MetricType::Counter)
                .with_help("Authorized unaccess to thumbnail")
                .build()
                .render_and_append_instance(
                    &PrometheusInstance::new().with_value(self.unauthorized_thumb),
                )
                .render(),
        );

        s
    }
}
