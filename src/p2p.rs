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
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

#[derive(NetworkBehaviour)]
struct PappapBehaviour {
    gossipsub: gossipsub::Behaviour,
    identify: libp2p::identify::Behaviour,
}

pub struct P2PNode {
    swarm: Swarm<PappapBehaviour>,
    topic: gossipsub::IdentTopic,
    pub peer_count: Arc<AtomicUsize>,
}

impl P2PNode {
    // S?A L?I: Hàm new nh?n thêm peer_count
    pub async fn new(local_key: identity::Keypair, peer_count: Arc<AtomicUsize>) -> Result<(Self, mpsc::UnboundedReceiver<Vec<u8>>, PeerId), Box<dyn Error>> {
        let local_peer_id = PeerId::from(local_key.public());
        
        let _transport = tcp::tokio::Transport::default()
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

        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(local_key.clone())
            .with_tokio()
            .with_tcp(tcp::Config::default(), noise::Config::new, yamux::Config::default)?
            .with_behaviour(|_| behaviour)?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        swarm.listen_on("/ip4/0.0.0.0/tcp/9000".parse()?)?;

        let (_, rx) = mpsc::unbounded_channel();
        Ok((Self { swarm, topic, peer_count }, rx, local_peer_id))
    }

    pub fn dial(&mut self, addr: &str) {
        if let Ok(ma) = Multiaddr::from_str(addr) {
            if let Err(e) = self.swarm.dial(ma) { println!("? Dial failed: {:?}", e); }
        }
    }
    pub fn broadcast_block(&mut self, block_data: Vec<u8>) {
        let _ = self.swarm.behaviour_mut().gossipsub.publish(self.topic.clone(), block_data);
    }
    pub async fn run(&mut self) {
        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::NewListenAddr { address, .. } => println!("?? P2P: {}", address),
                SwarmEvent::ConnectionEstablished { .. } => { self.peer_count.fetch_add(1, Ordering::Relaxed); },
                SwarmEvent::ConnectionClosed { .. } => { self.peer_count.fetch_sub(1, Ordering::Relaxed); },
                _ => {}
            }
        }
    }
}
