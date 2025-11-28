// src/network/p2p.rs
use libp2p::{
    gossipsub, identity, mdns, noise, tcp, yamux,
    swarm::{NetworkBehaviour, SwarmEvent},
    PeerId, Swarm,
};
use libp2p::futures::StreamExt;
use std::time::Duration;
use tracing::{info, warn};
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::{utils::{bus::GlobalBus, crypto::CryptoEngine}, core::snn::SNNCore};

#[derive(NetworkBehaviour)]
struct PappapBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
    keep_alive: libp2p::ping::Behaviour,
}

pub struct P2PNode {
    swarm: Swarm<PappapBehaviour>,
    bus_rx: broadcast::Receiver<(String, Vec<u8>)>, 
    bus_tx: GlobalBus,
    crypto: Arc<CryptoEngine>,
    snn: Arc<SNNCore>,
	port: u16,
}

impl P2PNode {
    pub async fn new(
        bus: GlobalBus,
        crypto: Arc<CryptoEngine>,
        snn: Arc<SNNCore>,
		port: u16,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        
        // 1. Tạo Identity
        let local_key = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(local_key.public());
        info!("Local Peer ID: {}", peer_id);

        // 2. Tạo Swarm bằng Builder Pattern chuẩn của libp2p 0.53
        // Code này thay thế hoàn toàn đoạn transport/upgrade cũ gây lỗi
        let swarm = libp2p::SwarmBuilder::with_existing_identity(local_key.clone())
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|key| {
                // Config Gossipsub
                let topic = gossipsub::IdentTopic::new("pappap-mainnet-v1");
                let gossip_config = gossipsub::ConfigBuilder::default()
                    .heartbeat_interval(Duration::from_secs(5))
                    .validation_mode(gossipsub::ValidationMode::Strict)
                    .build()
                    .expect("Valid config"); // Expect ở đây an toàn vì config tĩnh

                let mut gossip_behaviour = gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossip_config,
                ).expect("Correct config");
                
                gossip_behaviour.subscribe(&topic).unwrap();

                // Return struct Behaviour đã kết hợp
                PappapBehaviour {
                    gossipsub: gossip_behaviour,
                    mdns: mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id).expect("mDNS start"),
                    keep_alive: libp2p::ping::Behaviour::new(libp2p::ping::Config::new()),
                }
            })?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        Ok(Self {
            swarm,
            bus_rx: bus.subscribe().0, 
            bus_tx: bus,
            crypto,
            snn,
			port,
        })
    }

    pub async fn run(mut self) {
        self.swarm.listen_on("/ip4/0.0.0.0/tcp/9000".parse().unwrap()).unwrap();
		let addr_str = format!("/ip4/0.0.0.0/tcp/{}", self.port);
        self.swarm.listen_on(addr_str.parse().unwrap()).unwrap();
        
        loop {
            tokio::select! {
                Ok((target, data)) = self.bus_rx.recv() => {
                    if target == "broadcast" {
                        let encrypted = self.crypto.encrypt(&data);
                        let topic = gossipsub::IdentTopic::new("pappap-mainnet-v1");
                        let _ = self.swarm.behaviour_mut().gossipsub.publish(topic, encrypted);
						
                    }
                }
                
                event = self.swarm.select_next_some() => match event {
                    SwarmEvent::Behaviour(PappapBehaviourEvent::Gossipsub(gossipsub::Event::Message { message, .. })) => {
                        if self.snn.threat_check(message.data.len()).await {
                            warn!("Dropped potential threat packet");
                            continue;
                        }

                        if let Ok(decrypted) = self.crypto.decrypt(&message.data) {
                            self.bus_tx.publish("p2p_inbound", decrypted); 
                        }
                    },
                    SwarmEvent::NewListenAddr { address, .. } => info!("P2P Listening on {:?}", address),
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => info!("Connected to peer: {}", peer_id),
                    _ => {}
                }
            }
        }
    }
}