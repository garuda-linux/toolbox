use super::{DnsProvider, WifiBackend};

#[test]
fn every_provider_but_the_default_has_one_server() {
    assert_eq!(DnsProvider::Default.servers().len(), 0);
    assert_eq!(DnsProvider::Google.servers(), ["8.8.8.8"]);
    assert_eq!(DnsProvider::Cloudflare.servers(), ["1.1.1.1"]);
    assert_eq!(DnsProvider::Quad9.servers(), ["9.9.9.9"]);
}

#[test]
fn a_server_names_its_provider() {
    assert_eq!(
        DnsProvider::from_server("1.1.1.1"),
        Some(DnsProvider::Cloudflare)
    );
    assert_eq!(DnsProvider::from_server("192.168.1.1"), None);
}

#[test]
fn the_dns_file_holds_a_global_domain_section() {
    assert_eq!(
        DnsProvider::Quad9.config().as_deref(),
        Some("[global-dns-domain-*]\nservers=9.9.9.9\n")
    );
    assert_eq!(DnsProvider::Default.config(), None);
}

#[test]
fn ids_round_trip_and_match_serde() {
    for provider in DnsProvider::ALL {
        assert_eq!(
            serde_json::to_string(&provider).unwrap(),
            format!("\"{}\"", provider.as_str())
        );
    }

    for backend in WifiBackend::ALL {
        assert_eq!(
            serde_json::to_string(&backend).unwrap(),
            format!("\"{}\"", backend.as_str())
        );
    }
}

#[test]
fn only_iwd_needs_a_backend_file() {
    assert_eq!(
        WifiBackend::Iwd.config(),
        Some("[device]\nwifi.backend=iwd\n")
    );
    assert_eq!(WifiBackend::WpaSupplicant.config(), None);
}
