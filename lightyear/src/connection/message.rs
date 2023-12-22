/*!
Provides a [`ProtocolMessage`] enum that is a wrapper around all the possible messages that can be sent over the network
*/
use crate::prelude::NetworkTarget;
use serde::{Deserialize, Serialize};
use tracing::{info_span, trace};

use crate::protocol::Protocol;
use crate::shared::ping::message::SyncMessage;
use crate::shared::replication::{ReplicationMessage, ReplicationMessageData};
use crate::utils::named::Named;

// pub enum InternalMessage<P: Protocol> {
//     InputMessage(InputMessage<P::Input>),
// }
//
// pub enum InternalReplication<P: Protocol> {
//     ShouldBePredicted(ShouldBePredicted),
//     ShouldBeInterpolated(ShouldBeInterpolated),
// }

// ClientMessages can include some extra Metadata
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ClientMessage<P: Protocol> {
    Message(P::Message, NetworkTarget),
    Replication(ReplicationMessage<P::Components, P::ComponentKinds>),
    // the reason why we include sync here instead of doing another MessageManager is so that
    // the sync messages can be added to packets that have other messages
    Sync(SyncMessage),
}

impl<P: Protocol> ClientMessage<P> {
    pub(crate) fn emit_send_logs(&self, channel_name: &str) {
        match self {
            ClientMessage::Message(message, _) => {
                let message_name = message.name();
                trace!(channel = ?channel_name, message = ?message_name, "Sending message");
                #[cfg(metrics)]
                metrics::increment_counter!("send_message", "channel" => channel_name, "message" => message_name);
            }
            ClientMessage::Replication(message) => {
                let _span = info_span!("send replication message", channel = ?channel_name, group_id = ?message.group_id);
                #[cfg(metrics)]
                metrics::increment_counter!("send_replication_actions");
                match &message.data {
                    ReplicationMessageData::Actions(m) => {
                        for (entity, actions) in &m.actions {
                            let _span = info_span!("send replication actions", ?entity);
                            if actions.spawn {
                                trace!("Send entity spawn");
                                #[cfg(metrics)]
                                metrics::increment_counter!("send_entity_spawn");
                            }
                            if actions.despawn {
                                trace!("Send entity despawn");
                                #[cfg(metrics)]
                                metrics::increment_counter!("send_entity_despawn");
                            }
                            if !actions.insert.is_empty() {
                                let components = actions
                                    .insert
                                    .iter()
                                    .map(|c| c.into())
                                    .collect::<Vec<P::ComponentKinds>>();
                                trace!(?components, "Sending component insert");
                                #[cfg(metrics)]
                                {
                                    for component in components {
                                        metrics::increment_counter!("send_component_insert", "component" => kind);
                                    }
                                }
                            }
                            if !actions.remove.is_empty() {
                                trace!(?actions.remove, "Sending component remove");
                                #[cfg(metrics)]
                                {
                                    for kind in actions.remove {
                                        metrics::increment_counter!("send_component_remove", "component" => kind);
                                    }
                                }
                            }
                            if !actions.updates.is_empty() {
                                let components = actions
                                    .updates
                                    .iter()
                                    .map(|c| c.into())
                                    .collect::<Vec<P::ComponentKinds>>();
                                trace!(?components, "Sending component update");
                                #[cfg(metrics)]
                                {
                                    for component in components {
                                        metrics::increment_counter!("send_component_update", "component" => kind);
                                    }
                                }
                            }
                        }
                    }
                    ReplicationMessageData::Updates(m) => {
                        for (entity, updates) in &m.updates {
                            let _span = info_span!("send replication updates", ?entity);
                            let components = updates
                                .iter()
                                .map(|c| c.into())
                                .collect::<Vec<P::ComponentKinds>>();
                            trace!(?components, "Sending component update");
                            #[cfg(metrics)]
                            {
                                for component in components {
                                    metrics::increment_counter!("send_component_update", "component" => kind);
                                }
                            }
                        }
                    }
                }
            }
            ClientMessage::Sync(message) => match message {
                SyncMessage::Ping(_) => {
                    trace!(channel = ?channel_name, "Sending ping");
                    #[cfg(metrics)]
                    metrics::increment_counter!("send_ping", "channel" => channel_name);
                }
                SyncMessage::Pong(_) => {
                    trace!(channel = ?channel_name, "Sending pong");
                    #[cfg(metrics)]
                    metrics::increment_counter!("send_pong", "channel" => channel_name);
                }
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerMessage<P: Protocol> {
    Message(P::Message),
    Replication(ReplicationMessage<P::Components, P::ComponentKinds>),
    // the reason why we include sync here instead of doing another MessageManager is so that
    // the sync messages can be added to packets that have other messages
    Sync(SyncMessage),
}

impl<P: Protocol> ServerMessage<P> {
    pub(crate) fn emit_send_logs(&self, channel_name: &str) {
        match self {
            ServerMessage::Message(message) => {
                let message_name = message.name();
                trace!(channel = ?channel_name, message = ?message_name, "Sending message");
                #[cfg(metrics)]
                metrics::increment_counter!("send_message", "channel" => channel_name, "message" => message_name);
            }
            ServerMessage::Replication(message) => {
                let _span = info_span!("send replication message", channel = ?channel_name, group_id = ?message.group_id);
                #[cfg(metrics)]
                metrics::increment_counter!("send_replication_actions");
                match &message.data {
                    ReplicationMessageData::Actions(m) => {
                        for (entity, actions) in &m.actions {
                            let _span = info_span!("send replication actions", ?entity);
                            if actions.spawn {
                                trace!("Send entity spawn");
                                #[cfg(metrics)]
                                metrics::increment_counter!("send_entity_spawn");
                            }
                            if actions.despawn {
                                trace!("Send entity despawn");
                                #[cfg(metrics)]
                                metrics::increment_counter!("send_entity_despawn");
                            }
                            if !actions.insert.is_empty() {
                                let components = actions
                                    .insert
                                    .iter()
                                    .map(|c| c.into())
                                    .collect::<Vec<P::ComponentKinds>>();
                                trace!(?components, "Sending component insert");
                                #[cfg(metrics)]
                                {
                                    for component in components {
                                        metrics::increment_counter!("send_component_insert", "component" => kind);
                                    }
                                }
                            }
                            if !actions.remove.is_empty() {
                                trace!(?actions.remove, "Sending component remove");
                                #[cfg(metrics)]
                                {
                                    for kind in actions.remove {
                                        metrics::increment_counter!("send_component_remove", "component" => kind);
                                    }
                                }
                            }
                            if !actions.updates.is_empty() {
                                let components = actions
                                    .updates
                                    .iter()
                                    .map(|c| c.into())
                                    .collect::<Vec<P::ComponentKinds>>();
                                trace!(?components, "Sending component update");
                                #[cfg(metrics)]
                                {
                                    for component in components {
                                        metrics::increment_counter!("send_component_update", "component" => kind);
                                    }
                                }
                            }
                        }
                    }
                    ReplicationMessageData::Updates(m) => {
                        for (entity, updates) in &m.updates {
                            let _span = info_span!("send replication updates", ?entity);
                            let components = updates
                                .iter()
                                .map(|c| c.into())
                                .collect::<Vec<P::ComponentKinds>>();
                            trace!(?components, "Sending component update");
                            #[cfg(metrics)]
                            {
                                for component in components {
                                    metrics::increment_counter!("send_component_update", "component" => kind);
                                }
                            }
                        }
                    }
                }
            }
            ServerMessage::Sync(message) => match message {
                SyncMessage::Ping(_) => {
                    trace!(channel = ?channel_name, "Sending ping");
                    #[cfg(metrics)]
                    metrics::increment_counter!("send_ping", "channel" => channel_name);
                }
                SyncMessage::Pong(_) => {
                    trace!(channel = ?channel_name, "Sending pong");
                    #[cfg(metrics)]
                    metrics::increment_counter!("send_pong", "channel" => channel_name);
                }
            },
        }
    }
}

// TODO: add frequency information
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ProtocolMessage<P: Protocol> {
    Message(P::Message),
    Replication(ReplicationMessage<P::Components, P::ComponentKinds>),
    // the reason why we include sync here instead of doing another MessageManager is so that
    // the sync messages can be added to packets that have other messages
    Sync(SyncMessage),
}

// TODO: another option is to add ClientMessage and ServerMessage to ProtocolMessage
// then we can keep the shared logic in connection.mod. We just lose 1 bit everytime...

impl<P: Protocol> ProtocolMessage<P> {
    pub(crate) fn emit_send_logs(&self, channel_name: &str) {
        match self {
            ProtocolMessage::Message(message) => {
                let message_name = message.name();
                trace!(channel = ?channel_name, message = ?message_name, "Sending message");
                #[cfg(metrics)]
                metrics::increment_counter!("send_message", "channel" => channel_name, "message" => message_name);
            }
            ProtocolMessage::Replication(message) => {
                let _span = info_span!("send replication message", channel = ?channel_name, group_id = ?message.group_id);
                #[cfg(metrics)]
                metrics::increment_counter!("send_replication_actions");
                match &message.data {
                    ReplicationMessageData::Actions(m) => {
                        for (entity, actions) in &m.actions {
                            let _span = info_span!("send replication actions", ?entity);
                            if actions.spawn {
                                trace!("Send entity spawn");
                                #[cfg(metrics)]
                                metrics::increment_counter!("send_entity_spawn");
                            }
                            if actions.despawn {
                                trace!("Send entity despawn");
                                #[cfg(metrics)]
                                metrics::increment_counter!("send_entity_despawn");
                            }
                            if !actions.insert.is_empty() {
                                let components = actions
                                    .insert
                                    .iter()
                                    .map(|c| c.into())
                                    .collect::<Vec<P::ComponentKinds>>();
                                trace!(?components, "Sending component insert");
                                #[cfg(metrics)]
                                {
                                    for component in components {
                                        metrics::increment_counter!("send_component_insert", "component" => kind);
                                    }
                                }
                            }
                            if !actions.remove.is_empty() {
                                trace!(?actions.remove, "Sending component remove");
                                #[cfg(metrics)]
                                {
                                    for kind in actions.remove {
                                        metrics::increment_counter!("send_component_remove", "component" => kind);
                                    }
                                }
                            }
                            if !actions.updates.is_empty() {
                                let components = actions
                                    .updates
                                    .iter()
                                    .map(|c| c.into())
                                    .collect::<Vec<P::ComponentKinds>>();
                                trace!(?components, "Sending component update");
                                #[cfg(metrics)]
                                {
                                    for component in components {
                                        metrics::increment_counter!("send_component_update", "component" => kind);
                                    }
                                }
                            }
                        }
                    }
                    ReplicationMessageData::Updates(m) => {
                        for (entity, updates) in &m.updates {
                            let _span = info_span!("send replication updates", ?entity);
                            let components = updates
                                .iter()
                                .map(|c| c.into())
                                .collect::<Vec<P::ComponentKinds>>();
                            trace!(?components, "Sending component update");
                            #[cfg(metrics)]
                            {
                                for component in components {
                                    metrics::increment_counter!("send_component_update", "component" => kind);
                                }
                            }
                        }
                    }
                }
            }
            ProtocolMessage::Sync(message) => match message {
                SyncMessage::Ping(_) => {
                    trace!(channel = ?channel_name, "Sending ping");
                    #[cfg(metrics)]
                    metrics::increment_counter!("send_ping", "channel" => channel_name);
                }
                SyncMessage::Pong(_) => {
                    trace!(channel = ?channel_name, "Sending pong");
                    #[cfg(metrics)]
                    metrics::increment_counter!("send_pong", "channel" => channel_name);
                }
            },
        }
    }

    // pub(crate) fn push_to_events(
    //     self,
    //     channel_kind: ChannelKind,
    //     events: &mut ConnectionEvents<P>,
    //     entity_map: &EntityMap,
    //     time_manager: &TimeManager,
    //     // tick of the remote when they sent that message. This means this message represents
    //     // the state of the remote world for this tick
    //     tick: Tick,
    // ) {
    //     let _span = trace_span!("receive");
    //     match self {
    //         ProtocolMessage::Message(message) => {
    //             events.push_message(channel_kind, message);
    //         }
    //         ProtocolMessage::Replication(replication) => match replication {
    //             ReplicationMessage::SpawnEntity(entity, components) => {
    //                 // convert the remote entity to the local before sending to events
    //                 // if we can't find the local entity, just use the remote
    //                 let local_entity = *entity_map.get_local(entity).unwrap_or(&entity);
    //                 events.push_spawn(local_entity);
    //                 for component in components {
    //                     let kind: P::ComponentKinds = (&component).into();
    //                     events.push_insert_component(local_entity, kind, tick);
    //                 }
    //             }
    //             ReplicationMessage::DespawnEntity(entity) => {
    //                 let local_entity = *entity_map.get_local(entity).unwrap_or(&entity);
    //                 events.push_despawn(local_entity);
    //             }
    //             ReplicationMessage::InsertComponent(entity, components) => {
    //                 let local_entity = *entity_map.get_local(entity).unwrap_or(&entity);
    //                 for component in components {
    //                     let kind: P::ComponentKinds = (&component).into();
    //                     events.push_insert_component(local_entity, kind, tick);
    //                 }
    //             }
    //             ReplicationMessage::RemoveComponent(entity, component_kinds) => {
    //                 let local_entity = *entity_map.get_local(entity).unwrap_or(&entity);
    //                 for component_kind in component_kinds {
    //                     events.push_remove_component(local_entity, component_kind, tick);
    //                 }
    //             }
    //             ReplicationMessage::EntityUpdate(entity, components) => {
    //                 let local_entity = *entity_map.get_local(entity).unwrap_or(&entity);
    //                 for component in components {
    //                     let kind: P::ComponentKinds = (&component).into();
    //                     events.push_update_component(local_entity, kind, tick);
    //                 }
    //             }
    //         },
    //         _ => {}
    //     }
    // }
}
