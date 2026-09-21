use crate::persistent_runtime::domain::RuntimeNamespaceId;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

const MULTIPLEXER_ID_BYTES: usize = 16;
const MULTIPLEXER_ID_HEX_LEN: usize = MULTIPLEXER_ID_BYTES * 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ExactMultiplexerId([u8; MULTIPLEXER_ID_BYTES]);

impl ExactMultiplexerId {
    fn from_entropy_bytes(bytes: [u8; MULTIPLEXER_ID_BYTES]) -> Result<Self, String> {
        if bytes.iter().all(|byte| *byte == 0) {
            return Err("multiplexer identity entropy must not be all zero".to_owned());
        }
        Ok(Self(bytes))
    }

    fn parse(value: &str) -> Result<Self, String> {
        if value.len() != MULTIPLEXER_ID_HEX_LEN {
            return Err(format!(
                "multiplexer identity must be exactly {MULTIPLEXER_ID_HEX_LEN} lowercase hex characters"
            ));
        }
        if !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(
                "multiplexer identity must contain lowercase hexadecimal characters only"
                    .to_owned(),
            );
        }

        let mut bytes = [0_u8; MULTIPLEXER_ID_BYTES];
        for (index, chunk) in value.as_bytes().chunks_exact(2).enumerate() {
            let high = decode_hex_nibble(chunk[0])?;
            let low = decode_hex_nibble(chunk[1])?;
            bytes[index] = (high << 4) | low;
        }
        Self::from_entropy_bytes(bytes)
    }

    fn as_hex(self) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut output = String::with_capacity(MULTIPLEXER_ID_HEX_LEN);
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
        _ => Err("multiplexer identity contains non-lowercase-hex input".to_owned()),
    }
}

macro_rules! multiplexer_id_type {
    ($name:ident, $label:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub(crate) struct $name(ExactMultiplexerId);

        impl $name {
            pub(crate) fn from_entropy_bytes(
                bytes: [u8; MULTIPLEXER_ID_BYTES],
            ) -> Result<Self, String> {
                ExactMultiplexerId::from_entropy_bytes(bytes).map(Self)
            }

            pub(crate) fn parse(value: &str) -> Result<Self, String> {
                ExactMultiplexerId::parse(value).map(Self)
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

multiplexer_id_type!(MultiplexerWorkspaceId, "multiplexer workspace id");
multiplexer_id_type!(TabId, "tab id");
multiplexer_id_type!(PaneId, "pane id");
multiplexer_id_type!(LayoutTemplateId, "layout template id");
multiplexer_id_type!(AgentObservationId, "agent observation id");

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct TopologyGeneration(u64);

impl TopologyGeneration {
    pub(crate) const fn initial() -> Self {
        Self(1)
    }

    pub(crate) fn new(value: u64) -> Result<Self, String> {
        if value == 0 {
            return Err("topology generation must be greater than zero".to_owned());
        }
        Ok(Self(value))
    }

    pub(crate) const fn get(self) -> u64 {
        self.0
    }

    pub(crate) fn checked_next(self) -> Result<Self, MultiplexerErrorKind> {
        self.0
            .checked_add(1)
            .map(Self)
            .ok_or(MultiplexerErrorKind::TopologyGenerationExhausted)
    }
}

impl Serialize for TopologyGeneration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.0)
    }
}

impl<'de> Deserialize<'de> for TopologyGeneration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u64::deserialize(deserializer)?;
        Self::new(value).map_err(D::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum MultiplexerAuthority {
    Observer,
    MultiplexerWrite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum ClientSurfaceCapability {
    NonInteractiveObserver,
    ControllingTerminal,
    TrustedDesktopTerminalSurface,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum MultiplexerErrorKind {
    StaleTopologyGeneration,
    UnknownWorkspace,
    UnknownTab,
    UnknownPane,
    ClosedTarget,
    IdentityReuse,
    DuplicateOrReplayedMutation,
    SnapshotLimitExceeded,
    CapabilityUnavailable,
    MultiplexerWriteRequired,
    RuntimeControllerRequired,
    UnsupportedOperation,
    UnsupportedPlatform,
    OwnershipLost,
    OutcomeUnknown,
    TopologyGenerationExhausted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AgentObservationTopologyBinding {
    pub(crate) observation_id: AgentObservationId,
    pub(crate) multiplexer_workspace_id: MultiplexerWorkspaceId,
    pub(crate) tab_id: TabId,
    pub(crate) pane_id: PaneId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) runtime_namespace_id: Option<RuntimeNamespaceId>,
}

pub(crate) fn validate_replacement_pane_id(
    closed: PaneId,
    replacement: PaneId,
) -> Result<(), MultiplexerErrorKind> {
    if closed == replacement {
        return Err(MultiplexerErrorKind::IdentityReuse);
    }
    Ok(())
}

#[path = "navigation.rs"]
pub(crate) mod navigation;

#[path = "persistence.rs"]
pub(crate) mod persistence;

#[path = "service.rs"]
pub(crate) mod service;
