use crate::protocol::*;
use crate::shared::{shared_config, shared_movement_behaviour};
use crate::{Transports, KEY, PROTOCOL_ID};
use bevy::prelude::*;
use leafwing_input_manager::plugin::InputManagerSystem;
use leafwing_input_manager::prelude::*;
use leafwing_input_manager::systems::{run_if_enabled, tick_action_state};
use lightyear::_reexport::{ShouldBeInterpolated, ShouldBePredicted};
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::time::Duration;

#[derive(Resource, Clone)]
pub struct MyClientPlugin {
    pub(crate) client_id: ClientId,
    pub(crate) auth: Authentication,
    pub(crate) transport_config: TransportConfig,
}

pub(crate) fn create_plugin(
    client_id: u16,
    client_port: u16,
    server_addr: SocketAddr,
    transport: Transports,
) -> MyClientPlugin {
    let auth = Authentication::Manual {
        server_addr,
        client_id: client_id as ClientId,
        private_key: KEY,
        protocol_id: PROTOCOL_ID,
    };
    let client_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), client_port);
    let certificate_digest =
        String::from("c97a3b4c246684c77f694028d7b8eb40e25420b90b4c1165eb11a5423ebe5421");
    let transport_config = match transport {
        #[cfg(not(target_family = "wasm"))]
        Transports::Udp => TransportConfig::UdpSocket(client_addr),
        Transports::WebTransport => TransportConfig::WebTransportClient {
            client_addr,
            server_addr,
            #[cfg(target_family = "wasm")]
            certificate_digest,
        },
    };

    MyClientPlugin {
        client_id: client_id as ClientId,
        auth,
        transport_config,
    }
}

impl Plugin for MyClientPlugin {
    fn build(&self, app: &mut App) {
        let link_conditioner = LinkConditionerConfig {
            incoming_latency: Duration::from_millis(100),
            incoming_jitter: Duration::from_millis(10),
            incoming_loss: 0.00,
        };
        let io = Io::from_config(
            IoConfig::from_transport(self.transport_config.clone())
                .with_conditioner(link_conditioner),
        );
        let config = ClientConfig {
            shared: shared_config().clone(),
            input: InputConfig::default(),
            netcode: Default::default(),
            ping: PingConfig::default(),
            sync: SyncConfig::default(),
            prediction: PredictionConfig::default(),
            // we are sending updates every frame (60fps), let's add a delay of 6 network-ticks
            interpolation: InterpolationConfig::default().with_delay(
                InterpolationDelay::default()
                    .with_min_delay(Duration::from_millis(50))
                    .with_send_interval_ratio(2.0),
            ),
            // .with_delay(InterpolationDelay::Ratio(2.0)),
        };
        let plugin_config = PluginConfig::new(config, io, protocol(), self.auth.clone());
        app.add_plugins(ClientPlugin::new(plugin_config));
        app.add_plugins(crate::shared::SharedPlugin);
        // input-handling plugin from leafwing
        app.add_plugins(LeafwingInputPlugin::<MyProtocol, PlayerActions>::default());
        app.init_resource::<ActionState<PlayerActions>>();

        app.insert_resource(self.clone());
        app.add_systems(Startup, init);
        app.add_systems(FixedUpdate, movement.in_set(FixedUpdateSet::Main));
        app.add_systems(
            Update,
            (
                handle_player_spawn,
                send_message,
                handle_predicted_spawn,
                handle_interpolated_spawn,
                log,
            ),
        );
    }
}

// Startup system for the client
pub(crate) fn init(
    mut commands: Commands,
    mut client: ResMut<Client>,
    plugin: Res<MyClientPlugin>,
) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(TextBundle::from_section(
        format!("Client {}", plugin.client_id),
        TextStyle {
            font_size: 30.0,
            color: Color::WHITE,
            ..default()
        },
    ));
    client.connect();
    // client.set_base_relative_speed(0.001);
}

// The client input only gets applied to predicted entities that we own
// This works because we only predict the user's controlled entity.
// If we were predicting more entities, we would have to only apply movement to the player owned one.
pub(crate) fn movement(
    // TODO: maybe make prediction mode a separate component!!!
    mut position_query: Query<(&mut Position, &ActionState<PlayerActions>), With<Predicted>>,
) {
    // if we are not doing prediction, no need to read inputs
    if <Components as SyncMetadata<Position>>::mode() != ComponentSyncMode::Full {
        return;
    }
    for (position, action) in position_query.iter_mut() {
        shared_movement_behaviour(position, action);
    }
}

// System to receive messages on the client
pub(crate) fn send_message(mut client: ResMut<Client>) {
    // client.send_message::<DefaultUnorderedUnreliableChannel, _>(Message1(0));
    // info!("Send message");
}

// When the predicted copy of the client-owned entity is spawned, do stuff
// - assign it a different saturation
pub(crate) fn handle_player_spawn(
    mut commands: Commands,
    confirmed: Query<Entity, With<PlayerId>>,
) {
    for player_entity in confirmed.iter() {
        commands.entity(player_entity).insert(InputMap::new([
            (KeyCode::Right, PlayerActions::Right),
            (KeyCode::Left, PlayerActions::Left),
            (KeyCode::Up, PlayerActions::Up),
            (KeyCode::Down, PlayerActions::Down),
            (KeyCode::Delete, PlayerActions::Delete),
            (KeyCode::Space, PlayerActions::Spawn),
        ]));
    }
}

// When the predicted copy of the client-owned entity is spawned, do stuff
// - assign it a different saturation
pub(crate) fn handle_predicted_spawn(mut predicted: Query<&mut PlayerColor, Added<Predicted>>) {
    for mut color in predicted.iter_mut() {
        color.0.set_s(0.3);
    }
}

// When the predicted copy of the client-owned entity is spawned, do stuff
// - assign it a different saturation
pub(crate) fn handle_interpolated_spawn(
    mut interpolated: Query<&mut PlayerColor, Added<Interpolated>>,
) {
    for mut color in interpolated.iter_mut() {
        color.0.set_s(0.1);
    }
}

pub(crate) fn log(
    client: Res<Client>,
    confirmed: Query<&Position, With<Confirmed>>,
    predicted: Query<&Position, (With<Predicted>, Without<Confirmed>)>,
    mut interp_event: EventReader<ComponentInsertEvent<ShouldBeInterpolated>>,
    mut predict_event: EventReader<ComponentInsertEvent<ShouldBePredicted>>,
) {
    let server_tick = client.latest_received_server_tick();
    for confirmed_pos in confirmed.iter() {
        debug!(?server_tick, "Confirmed position: {:?}", confirmed_pos);
    }
    let client_tick = client.tick();
    for predicted_pos in predicted.iter() {
        debug!(?client_tick, "Predicted position: {:?}", predicted_pos);
    }
    for event in interp_event.read() {
        info!("Interpolated event: {:?}", event.entity());
    }
    for event in predict_event.read() {
        info!("Predicted event: {:?}", event.entity());
    }
}
