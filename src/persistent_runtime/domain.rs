use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

const PERSISTENT_ID_BYTES: usize = 16;
const PERSISTENT_ID_HEX_LEN: usize = PERSISTENT_ID_BYTES * 2;
const MAX_RUNTIME_ALIAS_BYTES: usize = 256;
const MAX_CLIENT_CONNECTION_ID_BYTES: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ExactPersistentId([u8; PERSISTENT_ID_BYTES]);

impl ExactPersistentId {
    fn from_entropy_bytes(bytes: [u8; PERSISTENT_ID_BYTES]) -> Result<Self, String> {
        if bytes.iter().all(|byte| *byte == 0) {
            return Err("persistent identity entropy must not be all zero".to_owned());
        }
        Ok(Self(bytes))
    }

    fn parse(value: &str) -> Result<Self, String> {
        if value.len() != PERSISTENT_ID_HEX_LEN {
            return Err(format!(
                "persistent identity must be exactly {PERSISTENT_ID_HEX_LEN} lowercase hex characters"
            ));
        }
        if !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(
                "persistent identity must contain lowercase hexadecimal characters only".to_owned(),
            );
        }

        let mut bytes = [0_u8; PERSISTENT_ID_BYTES];
        for (index, chunk) in value.as_bytes().chunks_exact(2).enumerate() {
            let high = decode_hex_nibble(chunk[0])?;
            let low = decode_hex_nibble(chunk[1])?;
            bytes[index] = (high << 4) | low;
        }
        Self::from_entropy_bytes(bytes)
    }

    fn as_hex(self) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut output = String::with_capacity(PERSISTENT_ID_HEX_LEN);
        for byte in self.0 {
            output.push(HEX[(byte >> 4) as usize] as char);
            output.push(HEX[(byte & 0x0f) as usize] as char);
        }
        output
    }
}

fn decode_hex_nibble(byte: u8) -> Result<u8, String> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err("persistent identity contains non-lowercase-hex input".to_owned()),
    }
}

macro_rules! persistent_id_type {
    ($name:ident, $label:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub(crate) struct $name(ExactPersistentId);

        impl $name {
            pub(crate) fn from_entropy_bytes(
                bytes: [u8; PERSISTENT_ID_BYTES],
            ) -> Result<Self, String> {
                ExactPersistentId::from_entropy_bytes(bytes).map(Self)
            }

            pub(crate) fn parse(value: &str) -> Result<Self, String> {
                ExactPersistentId::parse(value).map(Self)
            }

            pub(crate) fn as_hex(self) -> String {
                self.0.as_hex()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0.as_hex())
            }
        }

        impl FromStr for $name {
            type Err = String;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::parse(value)
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&self.0.as_hex())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::parse(&value)
                    .map_err(|error| D::Error::custom(format!("{}: {error}", $label)))
            }
        }
    };
}

persistent_id_type!(RuntimeNamespaceId, "runtime namespace id");
persistent_id_type!(OwnerGenerationId, "owner generation id");

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct RuntimeAlias(String);

impl RuntimeAlias {
    pub(crate) fn new(value: &str) -> Result<Self, String> {
        normalize_bounded_label(value, "runtime alias", MAX_RUNTIME_ALIAS_BYTES).map(Self)
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl Serialize for RuntimeAlias {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for RuntimeAlias {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(&value).map_err(D::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ClientConnectionId(String);

impl ClientConnectionId {
    pub(crate) fn new(value: &str) -> Result<Self, String> {
        normalize_bounded_label(
            value,
            "client connection id",
            MAX_CLIENT_CONNECTION_ID_BYTES,
        )
        .map(Self)
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl Serialize for ClientConnectionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for ClientConnectionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(&value).map_err(D::Error::custom)
    }
}

fn normalize_bounded_label(value: &str, label: &str, max_bytes: usize) -> Result<String, String> {
    if value.contains('\0') {
        return Err(format!("{label} must not contain NUL"));
    }
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(format!("{label} must not be empty"));
    }
    if normalized.len() > max_bytes {
        return Err(format!("{label} must be <= {max_bytes} UTF-8 bytes"));
    }
    Ok(normalized.to_owned())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum OwnershipState {
    LiveOwned,
    OwnershipLost,
    Unowned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum ProcessLiveness {
    Unknown,
    Running,
    Exited,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum EndpointAvailability {
    Unknown,
    Available,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum ContinuityClass {
    RetainedLiveProcess,
    ProviderNativeResume,
    WindsReconstruction,
    FreshProcess,
    Unknown,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum ClientAuthority {
    Observer,
    Controller,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum LifecycleProofClass {
    AgentReported,
    WindsObserved,
    HumanDecided,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum RuntimeLifecycleEventKind {
    NamespaceCreated,
    OwnershipEstablished,
    OwnershipLost,
    ProcessStateObserved,
    ContinuityClassified,
    ControllerChanged,
    RuntimeStopped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum LocalControlErrorKind {
    ProtocolMismatch,
    PrincipalDenied,
    StaleOwnerGeneration,
    UnknownRuntime,
    OwnershipLost,
    ControllerConflict,
    MalformedFrame,
    OversizedFrame,
    DuplicateOrOutOfOrderRequest,
    SlowClientBackpressure,
    UnsupportedOperation,
    UnsupportedPlatform,
    OutcomeUnknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EventSequence(u64);

impl EventSequence {
    pub(crate) fn new(value: u64) -> Result<Self, String> {
        if value == 0 {
            return Err("event sequence must be greater than zero".to_owned());
        }
        Ok(Self(value))
    }

    pub(crate) fn get(self) -> u64 {
        self.0
    }
}

impl Serialize for EventSequence {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.0)
    }
}

impl<'de> Deserialize<'de> for EventSequence {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u64::deserialize(deserializer)?;
        Self::new(value).map_err(D::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RuntimeTruth {
    pub(crate) ownership: OwnershipState,
    pub(crate) process_liveness: ProcessLiveness,
    pub(crate) endpoint_availability: EndpointAvailability,
    pub(crate) continuity: ContinuityClass,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ControllerLeaseIdentity {
    pub(crate) owner_generation_id: OwnerGenerationId,
    pub(crate) runtime_namespace_id: RuntimeNamespaceId,
    pub(crate) controller_client_id: ClientConnectionId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RuntimeLifecycleEvent {
    pub(crate) runtime_namespace_id: RuntimeNamespaceId,
    pub(crate) owner_generation_id: OwnerGenerationId,
    pub(crate) sequence: EventSequence,
    pub(crate) kind: RuntimeLifecycleEventKind,
    pub(crate) proof_class: LifecycleProofClass,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) controller_client_id: Option<ClientConnectionId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) observed_unix_ms: Option<u64>,
}
