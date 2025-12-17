use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
enum ListenConfiguration {
    #[serde(rename = "tcp")]
    Tcp(SocketAddr),
    #[serde(rename = "socket")]
    Socket(PathBuf),
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Default)]
enum LogLevelConfiguration {
    #[serde(rename = "debug")]
    Debug,
    #[serde(rename = "info")]
    #[default]
    Info,
    #[serde(rename = "warn")]
    Warn,
    #[serde(rename = "error")]
    Error,
}

#[derive(Debug, Serialize, Deserialize)]
struct ScriptConfiguration {
    #[serde(rename = "name")]
    name: String,
    #[serde(rename = "allowed_variables")]
    allowed_variables: Vec<String>,
    #[serde(rename = "working_directory")]
    working_directory: PathBuf,
    #[serde(rename = "command")]
    command: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClientConfiguration {
    #[serde(rename = "name")]
    name: String,
    #[serde(rename = "secret")]
    secret: String,
    #[serde(rename = "whitelisted_ips")]
    whitelisted_ips: Option<Vec<IpAddr>>,
    #[serde(rename = "blacklisted_ips")]
    blacklisted_ips: Option<Vec<IpAddr>>,
    #[serde(rename = "scripts")]
    scripts: Vec<ScriptConfiguration>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Configuration {
    #[serde(rename = "listen")]
    listen: ListenConfiguration,
    #[serde(rename = "log_level", default)]
    log_level: LogLevelConfiguration,
    #[serde(rename = "whitelisted_ips")]
    ip_whitelist: Option<Vec<IpAddr>>,
    #[serde(rename = "blacklisted_ips")]
    ip_blacklist: Option<Vec<IpAddr>>,
    #[serde(rename = "clients")]
    clients: Vec<ClientConfiguration>,
}

#[cfg(test)]
mod tests {
    use crate::configuration::ListenConfiguration::{Socket, Tcp};
    use crate::configuration::{Configuration, ListenConfiguration, LogLevelConfiguration};
    use std::fs::File;
    use std::net::IpAddr;
    use std::path::PathBuf;

    #[test]
    fn listen_configuration_tcp_ipv4_deserialization() {
        let yaml = r#"tcp: "0.0.0.0:8081""#;
        let configuration: ListenConfiguration = serde_saphyr::from_str(yaml).unwrap();

        let Tcp(addr) = configuration else {
            panic!("Expected Tcp configuration");
        };
        assert_eq!(addr.ip(), IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)));
        assert_eq!(addr.port(), 8081);
    }

    #[test]
    fn listen_configuration_tcp_ipv6_deserialization() {
        let yaml = r#"tcp: "[fc00:db20:35b:7399::5]:8081""#;
        let configuration: ListenConfiguration = serde_saphyr::from_str(yaml).unwrap();

        let Tcp(addr) = configuration else {
            panic!("Expected Tcp configuration");
        };
        assert_eq!(
            addr.ip(),
            IpAddr::V6(std::net::Ipv6Addr::new(
                0xfc00, 0xdb20, 0x35b, 0x7399, 0, 0, 0, 0x5
            ))
        );
        assert_eq!(addr.port(), 8081);
    }

    #[test]
    #[should_panic(expected = "invalid socket address syntax")]
    fn listen_configuration_tcp_malformed_deserialization() {
        let yaml = r#"tcp: "test:01""#;
        let _: ListenConfiguration = serde_saphyr::from_str(yaml).unwrap();
    }

    #[test]
    fn listen_configuration_socket_deserialization() {
        let yaml = r#"socket: "/tmp/socket""#;
        let configuration: ListenConfiguration = serde_saphyr::from_str(yaml).unwrap();
        let Socket(path) = configuration else {
            panic!("Expected Socket configuration");
        };
        assert_eq!(path, PathBuf::from("/tmp/socket"));
    }

    #[test]
    fn read_full_config_from_file() {
        let file = File::open("config.example.yaml").unwrap();
        let reader = std::io::BufReader::new(file);
        let configuration: Configuration = serde_saphyr::from_reader(reader).unwrap();
        let Tcp(addr) = configuration.listen else {
            panic!("Expected Tcp configuration");
        };
        assert_eq!(addr.ip(), IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)));
        assert_eq!(addr.port(), 8081);
        assert_eq!(configuration.log_level, LogLevelConfiguration::Error);
        assert_eq!(configuration.clients.len(), 1);
        assert_eq!(configuration.clients[0].name, "my-client");
        assert_eq!(configuration.clients[0].scripts.len(), 1);
        assert_eq!(configuration.clients[0].scripts[0].name, "my-script");
        assert_eq!(
            configuration.clients[0].scripts[0].allowed_variables.len(),
            1
        );
        assert_eq!(
            configuration.clients[0].scripts[0].allowed_variables[0],
            "MY_VAR"
        );
        assert_eq!(
            configuration.clients[0].scripts[0].working_directory,
            PathBuf::from("/tmp")
        );
        assert_eq!(configuration.clients[0].scripts[0].command.len(), 2);
        assert_eq!(configuration.clients[0].scripts[0].command[0], "echo");
        assert_eq!(configuration.clients[0].scripts[0].command[1], "{{MY_VAR}}");
        assert_eq!(configuration.clients[0].secret, "my-secret");
        assert_eq!(
            configuration.clients[0].whitelisted_ips,
            Some(vec![IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))])
        );
        assert_eq!(
            configuration.clients[0].blacklisted_ips,
            Some(vec![IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 0, 1))])
        );
        assert_eq!(
            configuration.ip_whitelist,
            Some(vec![IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))])
        );
        assert_eq!(
            configuration.ip_blacklist,
            Some(vec![IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 0, 1))])
        );
    }
}
