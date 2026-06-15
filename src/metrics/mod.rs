use metrics::{counter, describe_counter, describe_histogram, histogram};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};

pub fn init_metrics() -> PrometheusHandle {
    describe_counter!(
        "http_requests_total",
        "Total number of HTTP requests by method and path"
    );
    describe_histogram!(
        "http_request_duration_seconds",
        "HTTP request duration in seconds"
    );
    describe_counter!("auth_otp_sent_total", "Total OTP codes sent");
    describe_counter!("blocks_created_total", "Total blocks created");
    describe_counter!("queue_messages_total", "Total messages published to queue");

    PrometheusBuilder::new()
        .install_recorder()
        .expect("Failed to install Prometheus recorder")
}

pub fn record_request(method: &str, path: &str, status: u16, duration_secs: f64) {
    counter!(
        "http_requests_total",
        "method" => method.to_string(),
        "path" => path.to_string(),
        "status" => status.to_string()
    )
    .increment(1);
    histogram!(
        "http_request_duration_seconds",
        "method" => method.to_string(),
        "path" => path.to_string()
    )
    .record(duration_secs);
}

pub fn record_otp_sent() {
    counter!("auth_otp_sent_total").increment(1);
}

pub fn record_block_created() {
    counter!("blocks_created_total").increment(1);
}

pub fn record_queue_message(event_type: &str) {
    counter!("queue_messages_total", "type" => event_type.to_string()).increment(1);
}
