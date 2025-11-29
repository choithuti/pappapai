#!/bin/bash

echo "?? --- BU?C 1: D?NG M?I TH? & D?N D?P ---"
sudo systemctl stop pappap
sudo systemctl stop nginx
sudo killall -9 pappap-ai-chain 2>/dev/null
sudo fuser -k 8080/tcp 2>/dev/null
sudo fuser -k 9000/tcp 2>/dev/null

# Xóa Database cu d? tránh l?i Corruption
rm -rf /root/pappapai/pappap_data
echo "? Ðã xóa Database cu và d?ng ti?n trình."

echo "?? --- BU?C 2: C?P NH?T BACKEND RUST (PHIÊN B?N ?N Ð?NH) ---"
cd /root/pappapai

# 2.1 T?o Cargo.toml chu?n
cat > Cargo.toml << 'TOML'
[package]
name = "pappap-ai-chain"
version = "0.5.0"
edition = "2021"

[dependencies]
actix-web = "4"
actix-cors = "0.6"
actix-files = "0.6"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sysinfo = "0.29"
sled = "0.34"
bincode = "1.3"
chrono = "0.4"
sha2 = "0.10"
hex = "0.4"
rand = "0.8"
reqwest = { version = "0.11", default-features = false, features = ["json", "rustls-tls"] }
scraper = "0.13"
regex = "1.10"
libp2p = { version = "0.53", features = ["tcp", "noise", "yamux", "gossipsub", "identify", "macros", "tokio"] }
futures = "0.3"
env_logger = "0.10"
log = "0.4"
TOML

# 2.2 Build l?i b?n Release
echo "?? Ðang biên d?ch l?i (M?t kho?ng 1-2 phút)..."
/root/.cargo/bin/cargo build --release

if [ $? -ne 0 ]; then
    echo "? L?I BIÊN D?CH! D?ng l?i."
    exit 1
fi

echo "?? --- BU?C 3: CÀI Ð?T GIAO DI?N WEB 3D (MOBILE/PC) ---"
mkdir -p /var/www/html
cat > /var/www/html/index.html << 'HTML'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
    <title>PAPPAP AI | GENESIS NODE</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/three.js/r128/three.min.js"></script>
    <link href="https://fonts.googleapis.com/css2?family=Orbitron:wght@500;700;900&family=Share+Tech+Mono&display=swap" rel="stylesheet">
    <script>tailwind.config={theme:{extend:{colors:{bg:'#020204',cyan:'#00f3ff',pink:'#ff00aa',green:'#00ff41'},fontFamily:{display:['"Orbitron"'],mono:['"Share Tech Mono"']},boxShadow:{'neon':'0 0 10px rgba(0,243,255,0.4)'}}}}</script>
    <style>
        body{background:#020204;color:#e0f2fe;font-family:'Share Tech Mono',monospace;overflow-x:hidden}
        #canvas-bg{position:fixed;top:0;left:0;width:100%;height:100vh;z-index:-1}
        .glass{background:rgba(8,12,20,0.85);backdrop-filter:blur(10px);border:1px solid rgba(0,243,255,0.2);box-shadow:0 4px 30px rgba(0,0,0,0.5)}
        .tab-btn.active{color:#000;background:#00f3ff;font-weight:bold;box-shadow:0 0 15px rgba(0,243,255,0.4)}
        .hidden-tab{display:none}
    </style>
</head>
<body class="flex flex-col min-h-screen pb-6">
    <div id="canvas-bg"></div>
    <header class="h-16 border-b border-cyan/20 bg-black/90 backdrop-blur sticky top-0 z-50">
        <div class="max-w-7xl mx-auto px-4 h-full flex items-center justify-between">
            <div class="flex items-center gap-3">
                <div class="w-10 h-10 border-2 border-cyan flex justify-center items-center bg-black shadow-neon"><span class="text-2xl font-bold text-cyan animate-pulse">P</span></div>
                <div><h1 class="text-xl font-bold text-white tracking-widest">PAPPAP <span class="text-cyan">AI</span></h1><div class="text-[10px] text-pink tracking-[0.2em] font-bold">REBOOTED v0.6</div></div>
            </div>
            <div class="text-right"><div class="text-[9px] text-gray-400">HEIGHT</div><div class="text-lg text-cyan font-bold" id="head-height">#---</div></div>
        </div>
    </header>
    <main class="flex-1 max-w-7xl mx-auto w-full p-4 relative z-10">
        <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
            <div class="glass p-4 rounded text-center"><div class="text-[10px] text-gray-400">NEURONS</div><div class="text-2xl font-bold text-cyan" id="dash-neurons">---</div></div>
            <div class="glass p-4 rounded text-center"><div class="text-[10px] text-gray-400">POWER</div><div class="text-2xl font-bold text-pink" id="dash-power">---</div></div>
            <div class="glass p-4 rounded text-center"><div class="text-[10px] text-gray-400">STATUS</div><div class="text-xl font-bold text-green animate-pulse">ONLINE</div></div>
            <div class="glass p-4 rounded text-center"><div class="text-[10px] text-gray-400">PEERS</div><div class="text-2xl font-bold text-white">12</div></div>
        </div>
        <div class="glass p-4 rounded-xl mb-6"><h3 class="text-cyan font-bold text-sm mb-2">NEURAL CHAT</h3>
            <div id="chat-box" class="h-48 overflow-y-auto bg-black/40 p-3 rounded mb-2 text-sm border border-gray-700 space-y-2">
                <div class="text-gray-400">System: Connected to Pappap Core. Ask me anything.</div>
            </div>
            <div class="flex gap-2"><input type="text" id="chat-input" class="flex-1 bg-black border border-gray-600 p-2 rounded text-white focus:border-cyan outline-none" placeholder="Ask (e.g: Giá vàng)..." onkeydown="if(event.key==='Enter') sendChat()"><button onclick="sendChat()" class="bg-cyan text-black font-bold px-4 rounded">SEND</button></div>
        </div>
        <div class="glass p-4 rounded-xl"><h3 class="text-white font-bold text-sm mb-2">LATEST BLOCKS</h3><div id="blocks-log" class="text-xs font-mono space-y-1">Loading...</div></div>
    </main>
    <script>
        function init3D(){const c=document.getElementById('canvas-bg'),s=new THREE.Scene(),m=new THREE.PerspectiveCamera(75,window.innerWidth/window.innerHeight,0.1,1000),r=new THREE.WebGLRenderer({alpha:true});r.setSize(window.innerWidth,window.innerHeight);c.appendChild(r.domElement);const g=new THREE.BufferGeometry(),p=new Float32Array(900*3);for(let i=0;i<2700;i++)p[i]=(Math.random()-.5)*50;g.setAttribute('position',new THREE.BufferAttribute(p,3));const t=new THREE.PointsMaterial({size:0.15,color:0x00f3ff,transparent:true,opacity:0.6}),o=new THREE.Points(g,t);s.add(o);m.position.z=20;function a(){requestAnimationFrame(a);o.rotation.y+=.001;r.render(s,m)}a()}init3D();
        const API='/api';
        async function u(){try{const[h,b]=await Promise.all([fetch(API+'/health'),fetch(API+'/blocks')]),hd=await h.json(),bd=await b.json();document.getElementById('head-height').innerText='#'+hd.height;document.getElementById('dash-neurons').innerText=hd.neurons.toLocaleString();document.getElementById('dash-power').innerText=hd.power.toFixed(2);document.getElementById('blocks-log').innerHTML=bd.slice(0,8).map(x=>`<div class="flex justify-between border-b border-gray-800 py-1"><span class="text-cyan">#${x.index}</span><span class="text-gray-500 truncate w-32">${x.hash}</span><span class="text-pink">?${x.spike_score.toFixed(2)}</span></div>`).join('')}catch{}}
        async function sendChat(){const i=document.getElementById('chat-input'),v=i.value.trim();if(!v)return;const b=document.getElementById('chat-box');b.innerHTML+=`<div class="text-right text-white bg-gray-700 p-2 rounded inline-block ml-auto mb-2 max-w-[80%]">${v}</div>`;b.scrollTop=b.scrollHeight;i.value='';try{const r=await fetch(API+'/prompt',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({prompt:v})}),d=await r.json();b.innerHTML+=`<div class="text-left text-cyan bg-cyan/10 border border-cyan/30 p-2 rounded inline-block mr-auto mb-2 max-w-[80%]">${d.ai_response}</div>`;b.scrollTop=b.scrollHeight}catch{b.innerHTML+=`<div class="text-red-500">Error connecting.</div>`}}
        setInterval(u,1000);u();
    </script>
</body>
</html>
HTML

# C?p quy?n cho Nginx
chown -R www-data:www-data /var/www/html
chmod -R 755 /var/www/html

echo "?? --- BU?C 4: C?U HÌNH L?I NGINX & SSL ---"
cat > /etc/nginx/sites-available/pappap << 'NGINX'
server {
    listen 80;
    server_name pappapai.xyz www.pappapai.xyz;
    return 301 https://$host$request_uri;
}
server {
    listen 443 ssl;
    server_name pappapai.xyz www.pappapai.xyz;
    ssl_certificate /etc/letsencrypt/live/pappapai.xyz/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/pappapai.xyz/privkey.pem;
    
    location / {
        root /var/www/html;
        index index.html;
        try_files $uri $uri/ /index.html;
        add_header Cache-Control "no-cache, no-store, must-revalidate";
    }
    location /api/ {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
    }
}
NGINX

sudo systemctl restart nginx

echo "?? --- BU?C 5: KH?I Ð?NG L?I H? TH?NG ---"
sudo systemctl daemon-reload
sudo systemctl restart pappap

echo "?? HOÀN T?T! Hãy truy c?p https://pappapai.xyz (Nh? xóa Cache Ctrl+F5)"
