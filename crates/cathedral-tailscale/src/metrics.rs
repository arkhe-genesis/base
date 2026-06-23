use prometheus::{Counter, Gauge, Histogram, register_counter, register_gauge, register_histogram};

pub struct TailscaleMetrics {
    // Conectividade
    pub active_connections: Gauge,
    pub handshake_latency: Histogram,
    pub connection_duration: Histogram,

    // Tráfego
    pub bytes_sent: Counter,
    pub bytes_received: Counter,
    pub packets_dropped: Counter,

    // NAT/DERP
    pub nat_traversal_success: Counter,
    pub nat_traversal_failure: Counter,
    pub derp_fallback_count: Counter,
    pub derp_bytes_relayed: Counter,

    // Segurança
    pub psk_rotation_count: Counter,
    pub auth_failures: Counter,
    pub grants_violations: Counter,
}

impl TailscaleMetrics {
    pub fn new() -> Self {
        Self {
            active_connections: register_gauge!(
                "tailscale_active_connections",
                "Conexões ativas na tailnet"
            )
            .unwrap(),

            handshake_latency: register_histogram!(
                "tailscale_handshake_latency_seconds",
                "Latência do handshake WireGuard",
                vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0]
            )
            .unwrap(),

            connection_duration: register_histogram!(
                "tailscale_connection_duration_seconds",
                "Duração das conexões",
                vec![60.0, 300.0, 600.0, 1800.0, 3600.0]
            )
            .unwrap(),

            bytes_sent: register_counter!("tailscale_bytes_sent_total", "Bytes enviados").unwrap(),

            bytes_received: register_counter!("tailscale_bytes_received_total", "Bytes recebidos")
                .unwrap(),

            packets_dropped: register_counter!(
                "tailscale_packets_dropped_total",
                "Pacotes descartados"
            )
            .unwrap(),

            nat_traversal_success: register_counter!(
                "tailscale_nat_traversal_success_total",
                "NAT traversal bem-sucedido"
            )
            .unwrap(),

            nat_traversal_failure: register_counter!(
                "tailscale_nat_traversal_failure_total",
                "NAT traversal falhou → DERP"
            )
            .unwrap(),

            derp_fallback_count: register_counter!(
                "tailscale_derp_fallback_total",
                "Fallback para DERP"
            )
            .unwrap(),

            derp_bytes_relayed: register_counter!(
                "tailscale_derp_bytes_relayed_total",
                "Bytes relayados via DERP"
            )
            .unwrap(),

            psk_rotation_count: register_counter!(
                "tailscale_psk_rotation_total",
                "Rotações de PSK"
            )
            .unwrap(),

            auth_failures: register_counter!(
                "tailscale_auth_failures_total",
                "Falhas de autenticação"
            )
            .unwrap(),

            grants_violations: register_counter!(
                "tailscale_grants_violations_total",
                "Violações de Grants"
            )
            .unwrap(),
        }
    }
}

impl Default for TailscaleMetrics {
    fn default() -> Self {
        Self::new()
    }
}
