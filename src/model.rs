use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Protocol {
    Tcp,
    Udp,
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Tcp => write!(f, "TCP"),
            Protocol::Udp => write!(f, "UDP"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IpVersion {
    V4,
    V6,
}

impl std::fmt::Display for IpVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IpVersion::V4 => write!(f, "IPv4"),
            IpVersion::V6 => write!(f, "IPv6"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortBinding {
    pub port: u16,
    pub protocol: Protocol,
    pub ip_version: IpVersion,
    pub interface: String,
    pub is_all_interfaces: bool,
    pub is_localhost: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

impl PortBinding {
    pub fn key(&self) -> String {
        format!("{}:{}:{}", self.protocol, self.ip_version, self.port)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub created_at: String,
    pub hostname: String,
    pub count: usize,
    pub bindings: Vec<PortBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub added: Vec<PortBinding>,
    pub removed: Vec<PortBinding>,
    pub changed: Vec<ChangedBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangedBinding {
    pub key: String,
    pub port: u16,
    pub protocol: Protocol,
    pub previous: PortBinding,
    pub current: PortBinding,
}

impl DiffResult {
    pub fn has_differences(&self) -> bool {
        !self.added.is_empty() || !self.removed.is_empty() || !self.changed.is_empty()
    }

    pub fn has_new_exposures(&self) -> bool {
        self.added.iter().any(|b| b.is_all_interfaces)
    }
}
