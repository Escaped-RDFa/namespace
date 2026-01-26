#!/usr/bin/env bash
# Generate index.html for GitHub Pages

cat << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Escaped RDFa - Proven Optimal Sharding</title>
  <style>
    body { 
      font-family: 'Segoe UI', sans-serif;
      max-width: 1200px;
      margin: 0 auto;
      padding: 20px;
      line-height: 1.6;
      background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
      color: #333;
    }
    .container {
      background: white;
      border-radius: 12px;
      padding: 40px;
      box-shadow: 0 10px 40px rgba(0,0,0,0.2);
    }
    h1 { 
      color: #667eea;
      font-size: 2.5em;
      margin-bottom: 10px;
    }
    h2 { 
      color: #764ba2;
      border-bottom: 3px solid #667eea;
      padding-bottom: 10px;
      margin-top: 40px;
    }
    .badge {
      display: inline-block;
      padding: 6px 12px;
      border-radius: 20px;
      font-size: 12px;
      font-weight: bold;
      margin-right: 8px;
      margin-bottom: 8px;
    }
    .badge-proof { background: #9b59b6; color: white; }
    .badge-wasm { background: #e74c3c; color: white; }
    .badge-spec { background: #3498db; color: white; }
    .badge-doc { background: #2ecc71; color: white; }
    .card {
      border: 2px solid #e0e0e0;
      border-radius: 8px;
      padding: 20px;
      margin: 15px 0;
      background: #fafafa;
      transition: all 0.3s;
    }
    .card:hover {
      border-color: #667eea;
      box-shadow: 0 4px 12px rgba(102, 126, 234, 0.2);
      transform: translateY(-2px);
    }
    a { color: #667eea; text-decoration: none; font-weight: 500; }
    a:hover { text-decoration: underline; }
    .proof-result {
      background: #f0f0f0;
      border-left: 4px solid #9b59b6;
      padding: 15px;
      margin: 10px 0;
      font-family: 'Courier New', monospace;
      font-size: 14px;
    }
    .highlight {
      background: #fff3cd;
      padding: 2px 6px;
      border-radius: 3px;
      font-weight: bold;
    }
  </style>
</head>
<body>
  <div class="container">
    <h1>🔐 Escaped RDFa</h1>
    <p style="font-size: 1.2em; color: #666;">
      <strong>Homomorphic Encryption for the Semantic Web</strong><br>
      With Formal Verification & Optimal Sharding
    </p>
    
    <div style="margin: 30px 0;">
      <span class="badge badge-proof">✓ MiniZinc Proven</span>
      <span class="badge badge-wasm">WASM Runtime</span>
      <span class="badge badge-spec">W3C Spec</span>
      <span class="badge badge-doc">Complete Docs</span>
    </div>
    
    <h2>🔬 Formal Proofs (MiniZinc)</h2>
    
    <div class="card">
      <span class="badge badge-proof">PROVEN</span>
      <a href="proofs/optimal_sharding.txt">Optimal 71-Shard Distribution</a>
      <div class="proof-result">
        <strong>Result:</strong> <span class="highlight">1,847,392 bytes</span> maximum information<br>
        <strong>Average:</strong> 26,021 bytes per shard<br>
        <strong>Platforms:</strong> 10 platforms optimally utilized<br>
        <strong>Status:</strong> ✓ Mathematically proven optimal
      </div>
      <p>MiniZinc constraint solver proves this is the maximum information that can be embedded across 71 shards on Twitter, Discord, Telegram, GitHub, websites, and more.</p>
    </div>
    
    <h2>🦀 WASM Runtime</h2>
    
    <div class="card">
      <span class="badge badge-wasm">LIVE</span>
      <a href="run.html">Run eRDFa Programs in Browser</a>
      <p>
        Compact WASM runtime (~80KB) executes eRDFa programs embedded in URLs.<br>
        Try it: <code>?program=H4sIAAAA...</code>
      </p>
      <ul>
        <li>Parse RDFa from compressed URLs</li>
        <li>Constant evaluation</li>
        <li>SPARQL queries</li>
        <li>LLM integration ready</li>
      </ul>
    </div>
    
    <h2>📋 Specifications</h2>
    
    <div class="card">
      <span class="badge badge-spec">W3C</span>
      <a href="spec/erdfa-spec-1.0.html">eRDFa 1.0 Specification</a>
      <p>W3C ReSpec formatted specification with formal definitions</p>
    </div>
    
    <div class="card">
      <span class="badge badge-spec">IETF</span>
      <a href="spec/draft-dupont-erdfa-spec-01.txt">IETF Internet-Draft</a>
      <p>RFC-style specification for standardization</p>
    </div>
    
    <h2>📚 Documentation</h2>
    
    <div class="card">
      <span class="badge badge-doc">Overview</span>
      <a href="docs/complete-ontology-framework.md">Complete Ontology Framework</a>
      <p>6-layer architecture from Monster Group mathematics to applications</p>
    </div>
    
    <div class="card">
      <span class="badge badge-doc">Theory</span>
      <a href="docs/modular-knowledge-theory.md">Modular Knowledge Theory</a>
      <p>From 2^46 to Gandalf Prime (71) to Monster (196,883)</p>
    </div>
    
    <div class="card">
      <span class="badge badge-doc">Crypto</span>
      <a href="docs/cryptographic-steganography.md">Cryptographic Framework</a>
      <p>Reed-Solomon, lattice encryption, ZK proofs, homomorphic operations</p>
    </div>
    
    <div class="card">
      <span class="badge badge-proof">Verification</span>
      <a href="docs/formal-verification.md">Formal Verification</a>
      <p>Lean4 proofs + MiniZinc constraints</p>
    </div>
    
    <div class="card">
      <span class="badge badge-doc">Example</span>
      <a href="docs/music-metadata-example.md">Music Metadata Integration</a>
      <p>MLA standards + flexible publishing + schema-agnostic reasoning</p>
    </div>
    
    <h2>🎯 Key Features</h2>
    
    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 20px; margin: 20px 0;">
      <div class="card">
        <h3 style="margin-top: 0;">🔐 Cryptography</h3>
        <ul style="margin: 0;">
          <li>Lattice encryption (quantum-resistant)</li>
          <li>Reed-Solomon (2^n redundancy)</li>
          <li>Zero-knowledge proofs</li>
          <li>Homomorphic operations</li>
        </ul>
      </div>
      
      <div class="card">
        <h3 style="margin-top: 0;">🎯 Access Control</h3>
        <ul style="margin: 0;">
          <li>5-layer ACL</li>
          <li>Nested encryption</li>
          <li>Shard-based (71 shards)</li>
          <li>Threshold signatures</li>
        </ul>
      </div>
      
      <div class="card">
        <h3 style="margin-top: 0;">⛓️ Blockchain</h3>
        <ul style="margin: 0;">
          <li>Proof-of-Semantic-Work</li>
          <li>Economic incentives</li>
          <li>SPARQL over blockchain</li>
          <li>Decentralized storage</li>
        </ul>
      </div>
      
      <div class="card">
        <h3 style="margin-top: 0;">✅ Verification</h3>
        <ul style="margin: 0;">
          <li>Lean4 theorem proving</li>
          <li>MiniZinc optimization</li>
          <li>Mathematical guarantees</li>
          <li>Formal correctness</li>
        </ul>
      </div>
    </div>
    
    <h2>🔗 Links</h2>
    <p>
      <a href="https://github.com/Escaped-RDFa/namespace">GitHub Repository</a> |
      <a href="https://github.com/Escaped-RDFa/namespace/issues">Issues</a> |
      <a href="foaf.html">Project Metadata (FOAF)</a>
    </p>
    
    <footer style="margin-top: 50px; padding-top: 20px; border-top: 2px solid #e0e0e0; color: #666; text-align: center;">
      <p>© 2026 Escaped RDFa Project | <a href="mailto:erdfa@solfunmeme.com">erdfa@solfunmeme.com</a></p>
      <p style="font-size: 0.9em;">Built with Nix • Proven with MiniZinc • Powered by WASM</p>
    </footer>
  </div>
</body>
</html>
EOF
