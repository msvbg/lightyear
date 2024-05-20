//! Defines server-specific configuration options
use bevy::prelude::{Reflect, Resource};
use governor::Quota;
use nonzero_ext::nonzero;

use crate::connection::netcode::{Key, PRIVATE_KEY_BYTES};
use crate::connection::server::NetConfig;
use crate::shared::config::SharedConfig;
use crate::shared::ping::manager::PingConfig;

#[derive(Clone, Debug)]
pub struct NetcodeConfig {
    pub num_disconnect_packets: usize,
    pub keep_alive_send_rate: f64,
    /// Set the duration (in seconds) after which the server disconnects a client if they don't hear from them.
    /// This is valid for tokens generated by the server.
    /// The default is 3 seconds. A negative value means no timeout.
    pub client_timeout_secs: i32,
    pub protocol_id: u64,
    pub private_key: Key,
}

impl Default for NetcodeConfig {
    fn default() -> Self {
        Self {
            num_disconnect_packets: 10,
            keep_alive_send_rate: 1.0 / 10.0,
            client_timeout_secs: 3,
            protocol_id: 0,
            private_key: [0; PRIVATE_KEY_BYTES],
        }
    }
}

impl NetcodeConfig {
    pub fn with_protocol_id(mut self, protocol_id: u64) -> Self {
        self.protocol_id = protocol_id;
        self
    }
    pub fn with_key(mut self, key: Key) -> Self {
        self.private_key = key;
        self
    }

    pub fn with_client_timeout_secs(mut self, client_timeout_secs: i32) -> Self {
        self.client_timeout_secs = client_timeout_secs;
        self
    }
}

/// Configuration related to sending packets
#[derive(Clone, Debug)]
pub struct PacketConfig {
    /// After how many multiples of RTT do we consider a packet to be lost?
    ///
    /// The default is 1.5; i.e. after 1.5 times the round trip time, we consider a packet lost if
    /// we haven't received an ACK for it.
    pub nack_rtt_multiple: f32,
    /// Number of bytes per second that can be sent to each client
    pub per_client_send_bandwidth_cap: Quota,
    /// If false, there is no bandwidth cap and all messages are sent as soon as possible
    pub bandwidth_cap_enabled: bool,
}

impl Default for PacketConfig {
    fn default() -> Self {
        Self {
            nack_rtt_multiple: 1.5,
            // 56 KB/s bandwidth cap
            per_client_send_bandwidth_cap: Quota::per_second(nonzero!(56000u32)),
            bandwidth_cap_enabled: false,
        }
    }
}

impl PacketConfig {
    pub fn with_send_bandwidth_cap(mut self, send_bandwidth_cap: Quota) -> Self {
        self.per_client_send_bandwidth_cap = send_bandwidth_cap;
        self
    }

    pub fn with_send_bandwidth_bytes_per_second_cap(mut self, send_bandwidth_cap: u32) -> Self {
        let cap = send_bandwidth_cap.try_into().unwrap();
        self.per_client_send_bandwidth_cap = Quota::per_second(cap).allow_burst(cap);
        self
    }

    pub fn enable_bandwidth_cap(mut self) -> Self {
        self.bandwidth_cap_enabled = true;
        self
    }
}

#[derive(Clone, Debug, Default, Reflect)]
pub struct ReplicationConfig {
    /// By default, we will send all component updates since the last time we sent an update for a given entity.
    /// E.g. if the component was updated at tick 3; we will send the update at tick 3, and then at tick 4,
    /// we won't be sending anything since the component wasn't updated after that.
    ///
    /// This helps save bandwidth, but can cause the client to have delayed eventual consistency in the
    /// case of packet loss.
    ///
    /// If this is set to true, we will instead send all updates since the last time we received an ACK from the client.
    /// E.g. if the component was updated at tick 3; we will send the update at tick 3, and then at tick 4,
    /// we will send the update again even if the component wasn't updated, because we still haven't
    /// received an ACK from the client.
    pub send_updates_since_last_ack: bool,
}

/// Configuration for the server plugin
#[derive(Clone, Debug, Default, Resource)]
pub struct ServerConfig {
    pub shared: SharedConfig,
    /// The server can support multiple transport at the same time (e.g. UDP and WebTransport) so that
    /// clients can connect using the transport they prefer, and still play with each other!
    pub net: Vec<NetConfig>,
    pub packet: PacketConfig,
    pub replication: ReplicationConfig,
    pub ping: PingConfig,
}
