use libp2p::{
    gossipsub, identity, noise, tcp, yamux,
    swarm::{NetworkBehaviour, SwarmEvent},
    PeerId, Swarm, Transport, Multiaddr,
};
use libp2p::futures::StreamExt;
use std::time::Duration;
use tokio::sync::mpsc;
use std::error::Error;
use std::str::FromStr;

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
    pub async fn new(local_key: identity::Keypair) -> Result<(Self, mpsc::UnboundedReceiver<Vec<u8>>, PeerId), Box<dyn Error>> {
        let local_peer_id = PeerId::from(local_key.public());
        
        let transport = tcp::tokio::Transport::default()
            .upgrade(libp2p::core::upgrade::Version::V1Lazy)
            .authenticate(noise::Config::new(&local_key)?)
            .multiplex(yamux::Config::default())
            .boxed();

        let topic = gossipsub::IdentTopic::new("pappap-mainnet-blocks");
        let gossip_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(1))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .build()
            .map_err(|e| format!("Config error: {}", e)).unwrap();

        let mut behaviour = PappapBehaviour {
            gossipsub: gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(local_key.clone()),
                gossip_config,
            ).expect("Gossip init failed"),
            identify: libp2p::identify::Behaviour::new(
                libp2p::identify::Config::new("pappap/1.0.0".to_string(), local_key.public())
            ),
        };

        behaviour.gossipsub.subscribe(&topic).unwrap();

        // SỬA LỖI: Dùng libp2p::SwarmBuilder::with_tokio() để khớp với Tokio Runtime
        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(local_key.clone())
            .with_tokio()
            .with_tcp(tcp::Config::default(), noise::Config::new, yamux::Config::default)?
            .with_behaviour(|_| behaviour)?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();


        swarm.listen_on("/ip4/0.0.0.0/tcp/9000".parse()?)?;

        let (tx, rx) = mpsc::unbounded_channel();
        drop(tx); 

        Ok((Self { swarm, topic }, rx, local_peer_id))
    }

    pub fn dial(&mut self, addr: &str) {
        if let Ok(ma) = Multiaddr::from_str(addr) {
            if let Err(e) = self.swarm.dial(ma) { println!("❌ Dial failed: {:?}", e); }
        } else {
            println!("❌ Invalid address format: {}", addr);
        }
    }

    pub fn broadcast_block(&mut self, block_data: Vec<u8>) {
        let _ = self.swarm.behaviour_mut().gossipsub.publish(self.topic.clone(), block_data);
    }

    pub async fn run(&mut self) {
        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::NewListenAddr { address, .. } => {
                    let display_addr = address.to_string().replace("0.0.0.0", "72.61.126.190");
                    println!("🌐 P2P ACTIVE: {}", display_addr);
                }
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    println!("🤝 NEW PEER: {}", peer_id);
                }
                SwarmEvent::Behaviour(PappapBehaviourEvent::Gossipsub(gossipsub::Event::Message { message, .. })) => {
                    println!("📨 Received Data from {:?}", message.source);
                }
                _ => {}
            }
        }
    }
}
