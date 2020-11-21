use crate::file_type::FileType;
use crate::options::Options;
use prometheus_exporter_base::prelude::*;
use rocket::State;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[inline]
pub(crate) fn track_authorized_first_level_folders(
    options: &State<'_, Options>,
    statistics: &State<'_, Arc<RwLock<Statistics>>>,
) {
    if options.prometheus_metrics_enabled {
        statistics.write().unwrap().authorized_first_level_folders += 1;
    }
}

#[inline]
pub(crate) fn track_unauthorized_first_level_folders(
    options: &State<'_, Options>,
    statistics: &State<'_, Arc<RwLock<Statistics>>>,
) {
    if options.prometheus_metrics_enabled {
        statistics.write().unwrap().unauthorized_first_level_folders += 1;
    }
}

#[inline]
pub(crate) fn track_authorized_list_files(
    options: &State<'_, Options>,
    statistics: &State<'_, Arc<RwLock<Statistics>>>,
    file_tye: FileType,
) {
    // only keep track of the accesses if the
    // prometheus exporting has been
    // enabled!
    if options.prometheus_metrics_enabled {
        statistics
            .write()
            .unwrap()
            .inc_authorized_list_files(file_tye);
    }
}

#[inline]
pub(crate) fn track_unauthorized_list_files(
    options: &State<'_, Options>,
    statistics: &State<'_, Arc<RwLock<Statistics>>>,
    file_tye: FileType,
) {
    // only keep track of the accesses if the
    // prometheus exporting has been
    // enabled!
    if options.prometheus_metrics_enabled {
        statistics
            .write()
            .unwrap()
            .inc_unauthorized_list_files(file_tye);
    }
}

#[inline]
pub(crate) fn track_authorized_static(
    options: &State<'_, Options>,
    statistics: &State<'_, Arc<RwLock<Statistics>>>,
    path: &str,
) {
    // only keep track of the accesses if the
    // prometheus exporting has been
    // enabled!
    if options.prometheus_metrics_enabled {
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
    if options.prometheus_metrics_enabled {
        statistics.write().unwrap().authorized_dynamic += 1;
    }
}

#[inline]
pub(crate) fn track_unauthorized_dynamic(
    options: &State<'_, Options>,
    statistics: &State<'_, Arc<RwLock<Statistics>>>,
) {
    if options.prometheus_metrics_enabled {
        statistics.write().unwrap().unauthorized_dynamic += 1;
    }
}

#[inline]
pub(crate) fn track_picture_thumb_access(
    options: &State<'_, Options>,
    statistics: &State<'_, Arc<RwLock<Statistics>>>,
) {
    if options.prometheus_metrics_enabled {
        statistics.write().unwrap().picture_thumb_access += 1;
    }
}

#[inline]
pub(crate) fn track_picture_thumb_generation(
    options: &State<'_, Options>,
    statistics: &State<'_, Arc<RwLock<Statistics>>>,
) {
    if options.prometheus_metrics_enabled {
        statistics.write().unwrap().picture_thumb_generation += 1;
    }
}

#[inline]
pub(crate) fn track_video_thumb_access(
    options: &State<'_, Options>,
    statistics: &State<'_, Arc<RwLock<Statistics>>>,
) {
    if options.prometheus_metrics_enabled {
        statistics.write().unwrap().video_thumb_access += 1;
    }
}

#[inline]
pub(crate) fn track_video_thumb_generation(
    options: &State<'_, Options>,
    statistics: &State<'_, Arc<RwLock<Statistics>>>,
) {
    if options.prometheus_metrics_enabled {
        statistics.write().unwrap().video_thumb_generation += 1;
    }
}

#[inline]
pub(crate) fn track_unauthorized_static(
    options: &State<'_, Options>,
    statistics: &State<'_, Arc<RwLock<Statistics>>>,
    path: &str,
) {
    // only keep track of the accesses if the
    // prometheus exporting has been
    // enabled!
    if options.prometheus_metrics_enabled {
        statistics.write().unwrap().inc_unathorized_static(path);
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
    if options.prometheus_metrics_enabled {
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
    if options.prometheus_metrics_enabled {
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
    if options.prometheus_metrics_enabled {
        statistics.write().unwrap().authorized_thumb += 1;
    }
}

#[derive(Debug)]
pub struct Statistics {
    pub authorized_static: HashMap<String, u64>,
    pub unathorized_static: HashMap<String, u64>,
    pub unauthorized_dynamic: u64,
    pub authorized_dynamic: u64,
    pub authorized_not_found: u64,
    pub authorized_thumb: u64,
    pub unauthorized_thumb: u64,
    pub picture_thumb_access: u64,
    pub picture_thumb_generation: u64,
    pub video_thumb_access: u64,
    pub video_thumb_generation: u64,
    pub authorized_list_files: HashMap<FileType, u64>,
    pub unauthorized_list_files: HashMap<FileType, u64>,
    pub authorized_first_level_folders: u64,
    pub unauthorized_first_level_folders: u64,
}

impl Default for Statistics {
    fn default() -> Self {
        let mut authorized_list_files = HashMap::new();
        authorized_list_files.insert(FileType::Preview, 0);
        authorized_list_files.insert(FileType::Extra, 0);
        authorized_list_files.insert(FileType::Folder, 0);

        let mut unauthorized_list_files = HashMap::new();
        unauthorized_list_files.insert(FileType::Preview, 0);
        unauthorized_list_files.insert(FileType::Extra, 0);
        unauthorized_list_files.insert(FileType::Folder, 0);

        Self {
            authorized_static: HashMap::new(),
            unathorized_static: HashMap::new(),
            authorized_dynamic: 0,
            unauthorized_dynamic: 0,
            authorized_not_found: 0,
            authorized_thumb: 0,
            unauthorized_thumb: 0,
            picture_thumb_access: 0,
            picture_thumb_generation: 0,
            video_thumb_access: 0,
            video_thumb_generation: 0,
            authorized_list_files,
            unauthorized_list_files,
            authorized_first_level_folders: 0,
            unauthorized_first_level_folders: 0,
        }
    }
}

impl Statistics {
    pub(crate) fn inc_authorized_list_files(&mut self, file_type: FileType) {
        if let Some(original_value) = self.authorized_list_files.get_mut(&file_type) {
            *original_value += 1;
        } else {
            self.authorized_list_files.insert(file_type, 1);
        }
    }

    pub(crate) fn inc_unauthorized_list_files(&mut self, file_type: FileType) {
        if let Some(original_value) = self.unauthorized_list_files.get_mut(&file_type) {
            *original_value += 1;
        } else {
            self.unauthorized_list_files.insert(file_type, 1);
        }
    }

    pub(crate) fn inc_authorized_static(&mut self, page: &str) {
        if let Some(original_value) = self.authorized_static.get_mut(page) {
            *original_value += 1;
        } else {
            self.authorized_static.insert(page.to_owned(), 1);
        }
    }

    pub(crate) fn inc_unathorized_static(&mut self, page: &str) {
        if let Some(original_value) = self.unathorized_static.get_mut(page) {
            *original_value += 1;
        } else {
            self.unathorized_static.insert(page.to_owned(), 1);
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
            .with_name("nas_gallery_unauthorized_access_to_static_content")
            .with_metric_type(MetricType::Counter)
            .with_help("Unauthorized access to static content")
            .build();

        self.unathorized_static.iter().for_each(|(key, val)| {
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

        s.push_str(
            &PrometheusMetric::build()
                .with_name("nas_gallery_picture_thumb_access")
                .with_metric_type(MetricType::Counter)
                .with_help("Authorized unaccess picute thumb")
                .build()
                .render_and_append_instance(
                    &PrometheusInstance::new().with_value(self.picture_thumb_access),
                )
                .render(),
        );

        s.push_str(
            &PrometheusMetric::build()
                .with_name("nas_gallery_picture_thumb_generation")
                .with_metric_type(MetricType::Counter)
                .with_help("Picture thumb generation (cache miss)")
                .build()
                .render_and_append_instance(
                    &PrometheusInstance::new().with_value(self.picture_thumb_generation),
                )
                .render(),
        );

        s.push_str(
            &PrometheusMetric::build()
                .with_name("nas_gallery_video_thumb_access")
                .with_metric_type(MetricType::Counter)
                .with_help("Authorized unaccess picute thumb")
                .build()
                .render_and_append_instance(
                    &PrometheusInstance::new().with_value(self.video_thumb_access),
                )
                .render(),
        );

        s.push_str(
            &PrometheusMetric::build()
                .with_name("nas_gallery_video_thumb_generation")
                .with_metric_type(MetricType::Counter)
                .with_help("Video thumb generation (cache miss)")
                .build()
                .render_and_append_instance(
                    &PrometheusInstance::new().with_value(self.video_thumb_generation),
                )
                .render(),
        );

        let mut pc = PrometheusMetric::build()
            .with_name("nas_gallery_authorized_list_files")
            .with_metric_type(MetricType::Counter)
            .with_help("Authorized list files")
            .build();

        self.authorized_list_files.iter().for_each(|(key, val)| {
            pc.render_and_append_instance(
                &PrometheusInstance::new()
                    .with_label("file_type", key.as_str())
                    .with_value(*val),
            );
        });
        s.push_str(&pc.render());

        let mut pc = PrometheusMetric::build()
            .with_name("nas_gallery_unauthorized_list_files")
            .with_metric_type(MetricType::Counter)
            .with_help("Unauthorized list files")
            .build();

        self.unauthorized_list_files.iter().for_each(|(key, val)| {
            pc.render_and_append_instance(
                &PrometheusInstance::new()
                    .with_label("file_type", key.as_str())
                    .with_value(*val),
            );
        });
        s.push_str(&pc.render());

        s.push_str(
            &PrometheusMetric::build()
                .with_name("nas_gallery_authorized_first_level_folders")
                .with_metric_type(MetricType::Counter)
                .with_help("Authorized enumeration of first level folders")
                .build()
                .render_and_append_instance(
                    &PrometheusInstance::new().with_value(self.authorized_first_level_folders),
                )
                .render(),
        );

        s.push_str(
            &PrometheusMetric::build()
                .with_name("nas_gallery_unauthorized_first_level_folders")
                .with_metric_type(MetricType::Counter)
                .with_help("Unauthorized enumeration of first level folders")
                .build()
                .render_and_append_instance(
                    &PrometheusInstance::new().with_value(self.unauthorized_first_level_folders),
                )
                .render(),
        );

        s
    }
}
