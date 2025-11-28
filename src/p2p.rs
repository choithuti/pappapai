use libp2p::{
    gossipsub, identity, noise, tcp, yamux,
    swarm::{NetworkBehaviour, SwarmEvent},
    PeerId, Swarm, Transport,
};
use libp2p::futures::StreamExt;
use std::time::Duration;
use tokio::sync::mpsc;
use std::error::Error;

#[derive(NetworkBehaviour)]
struct PappapBehaviour {
    gossipsub: gossipsub::Behaviour,
    identify: libp2p::identify::Behaviour,
}

pub struct P2PNode {
    swarm: Swarm<PappapBehaviour>,
    topic: gossipsub::IdentTopic,
}

impl P2PNode {
    pub async fn new() -> Result<(Self, mpsc::UnboundedReceiver<Vec<u8>>, PeerId), Box<dyn Error>> {
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        println!("🆔 LOCAL PEER ID: {}", local_peer_id);

        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(local_key.clone())
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|key| {
                let topic = gossipsub::IdentTopic::new("pappap-mainnet-blocks");
                let gossip_config = gossipsub::ConfigBuilder::default()
                    .heartbeat_interval(Duration::from_secs(1))
                    .validation_mode(gossipsub::ValidationMode::Strict)
                    .build()
                    .map_err(|e| format!("Config error: {}", e)).unwrap();

                let mut gossip_behaviour = gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossip_config,
                ).expect("Gossip init failed");

                // Subscribe ngay trong behaviour setup
                gossip_behaviour.subscribe(&topic).unwrap();

                let identify_behaviour = libp2p::identify::Behaviour::new(
                    libp2p::identify::Config::new("pappap/1.0.0".to_string(), key.public())
                );

                PappapBehaviour {
                    gossipsub: gossip_behaviour,
                    identify: identify_behaviour,
                }
            })?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        swarm.listen_on("/ip4/0.0.0.0/tcp/9000".parse()?)?;

        let (tx, rx) = mpsc::unbounded_channel();
        drop(tx); 

        // Khai báo lại topic để lưu vào struct
        let topic = gossipsub::IdentTopic::new("pappap-mainnet-blocks");

        Ok((Self { swarm, topic }, rx, local_peer_id))
    }

    pub fn broadcast_block(&mut self, block_data: Vec<u8>) {
        if let Err(e) = self.swarm.behaviour_mut().gossipsub.publish(self.topic.clone(), block_data) {
            println!("❌ Publish error: {:?}", e);
        }
    }

    pub async fn run(&mut self) {
        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::NewListenAddr { address, .. } => {
                    let display_addr = address.to_string().replace("0.0.0.0", "72.61.126.190");
                    println!("🌐 P2P LISTENING: {}", address);
                    println!("🚀 BOOTNODE ADDRESS: {}/p2p/{}", display_addr, self.swarm.local_peer_id());
                }
                SwarmEvent::Behaviour(PappapBehaviourEvent::Gossipsub(gossipsub::Event::Message { message, .. })) => {
                    println!("📨 Received Block: {} bytes from {:?}", message.data.len(), message.source);
                }
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    println!("🤝 Peer Connected: {}", peer_id);
                }
                _ => {}
            }
        }
    }
}
