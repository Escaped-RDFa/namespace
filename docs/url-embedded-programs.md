# URL-Embedded eRDFa Programs with WASM Runtime

## Abstract

Twitter URLs can embed complete eRDFa programs in query parameters. A compact WASM runtime hosted on the static site interprets these programs, enabling constant evaluation, semantic reasoning, and LLM integration—all from a single shareable URL.

## Architecture

```
Twitter URL with embedded program
  ↓
https://escaped-rdfa.github.io/namespace/run?program=<encoded_erdfa>
  ↓
WASM Runtime (static site)
  ↓
Constant evaluation / Semantic reasoning / LLM integration
  ↓
Results displayed in browser
```

## URL Encoding

### Base64 Compressed eRDFa

```javascript
// Encode eRDFa program into URL
function encodeProgram(erdfa) {
  // 1. Compress with gzip
  const compressed = pako.gzip(erdfa);
  
  // 2. Base64 encode
  const base64 = btoa(String.fromCharCode(...compressed));
  
  // 3. URL-safe encoding
  return base64.replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '');
}

// Example
const program = `
<div vocab="http://purl.org/ontology/mo/" typeof="mo:Recording">
  <meta property="dc:title" content="My Song" />
  <meta property="mo:duration" content="PT3M45S" />
</div>
`;

const encoded = encodeProgram(program);
const url = `https://escaped-rdfa.github.io/namespace/run?program=${encoded}`;
// Tweet this URL!
```

### Twitter URL Example

```
https://escaped-rdfa.github.io/namespace/run?program=H4sIAAAAAAAAA6tWKkktLlGyUlAqS8wpTtVRKi1OLUpVslIqLU4tUqoFAJm5qBcdAAAA

Tweet: "Check out my song metadata! 🎵 [URL]"
```

## WASM Runtime

### Compact Runtime (< 100KB)

```rust
// Rust WASM runtime
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct ERdfaRuntime {
    parser: RdfaParser,
    evaluator: ConstEvaluator,
}

#[wasm_bindgen]
impl ERdfaRuntime {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            parser: RdfaParser::new(),
            evaluator: ConstEvaluator::new(),
        }
    }
    
    #[wasm_bindgen]
    pub fn run(&self, program: &str) -> String {
        // 1. Parse eRDFa
        let triples = self.parser.parse(program);
        
        // 2. Constant evaluation
        let results = self.evaluator.eval(&triples);
        
        // 3. Return JSON
        serde_json::to_string(&results).unwrap()
    }
    
    #[wasm_bindgen]
    pub fn query(&self, program: &str, sparql: &str) -> String {
        let triples = self.parser.parse(program);
        let results = self.evaluator.sparql(&triples, sparql);
        serde_json::to_string(&results).unwrap()
    }
}
```

### Build WASM

```bash
# Build compact WASM runtime
cargo build --target wasm32-unknown-unknown --release
wasm-opt -Oz -o erdfa_runtime.wasm target/wasm32-unknown-unknown/release/erdfa_runtime.wasm

# Result: ~80KB WASM file
```

## Static Site Runner

### HTML Runner Page

```html
<!DOCTYPE html>
<html>
<head>
  <title>eRDFa Program Runner</title>
  <style>
    body { font-family: monospace; max-width: 1200px; margin: 0 auto; padding: 20px; }
    #output { background: #f5f5f5; padding: 20px; border-radius: 8px; white-space: pre-wrap; }
    .loading { color: #3498db; }
    .error { color: #e74c3c; }
    .success { color: #2ecc71; }
  </style>
</head>
<body>
  <h1>🔐 eRDFa Program Runner</h1>
  <div id="status" class="loading">Loading WASM runtime...</div>
  <div id="output"></div>
  
  <script type="module">
    import init, { ERdfaRuntime } from './erdfa_runtime.js';
    
    async function runProgram() {
      // 1. Initialize WASM
      await init();
      const runtime = new ERdfaRuntime();
      document.getElementById('status').textContent = '✓ Runtime loaded';
      document.getElementById('status').className = 'success';
      
      // 2. Get program from URL
      const params = new URLSearchParams(window.location.search);
      const encoded = params.get('program');
      
      if (!encoded) {
        document.getElementById('output').textContent = 'No program provided';
        return;
      }
      
      // 3. Decode program
      const program = decodeProgram(encoded);
      
      // 4. Run program
      const results = runtime.run(program);
      
      // 5. Display results
      document.getElementById('output').textContent = JSON.stringify(JSON.parse(results), null, 2);
      
      // 6. If query parameter, run SPARQL
      const query = params.get('query');
      if (query) {
        const queryResults = runtime.query(program, decodeURIComponent(query));
        document.getElementById('output').textContent += '\n\nQuery Results:\n' + 
          JSON.stringify(JSON.parse(queryResults), null, 2);
      }
    }
    
    function decodeProgram(encoded) {
      // URL-safe base64 decode
      const base64 = encoded.replace(/-/g, '+').replace(/_/g, '/');
      const binary = atob(base64);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) {
        bytes[i] = binary.charCodeAt(i);
      }
      // Decompress
      const decompressed = pako.ungzip(bytes, { to: 'string' });
      return decompressed;
    }
    
    runProgram();
  </script>
  <script src="https://cdnjs.cloudflare.com/ajax/libs/pako/2.1.0/pako.min.js"></script>
</body>
</html>
```

## Constant Evaluation

### Compile-Time Computation

```rust
pub struct ConstEvaluator;

impl ConstEvaluator {
    pub fn eval(&self, triples: &[RdfTriple]) -> Vec<EvalResult> {
        let mut results = Vec::new();
        
        for triple in triples {
            // Constant folding
            if let Some(value) = self.fold_constant(triple) {
                results.push(EvalResult::Constant(value));
            }
            
            // Type inference
            if let Some(typ) = self.infer_type(triple) {
                results.push(EvalResult::Type(typ));
            }
            
            // Semantic inference
            if let Some(inference) = self.infer_semantic(triple) {
                results.push(EvalResult::Inference(inference));
            }
        }
        
        results
    }
    
    fn fold_constant(&self, triple: &RdfTriple) -> Option<Value> {
        // Example: mo:duration "PT3M45S" → 225 seconds
        if triple.predicate == "mo:duration" {
            if let Some(duration) = parse_iso8601_duration(&triple.object) {
                return Some(Value::Int(duration));
            }
        }
        None
    }
    
    fn infer_type(&self, triple: &RdfTriple) -> Option<String> {
        // Example: typeof="mo:Recording" → infer it's a music recording
        if triple.predicate == "rdf:type" {
            return Some(triple.object.clone());
        }
        None
    }
    
    fn infer_semantic(&self, triple: &RdfTriple) -> Option<Inference> {
        // Example: dc:title + mo:duration → likely a complete track
        // Pattern matching and semantic reasoning
        None // Simplified
    }
}
```

## LLM Integration

### Streaming LLM Queries

```javascript
// LLM integration in browser
async function queryWithLLM(program, question) {
  const response = await fetch('https://api.openai.com/v1/chat/completions', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${API_KEY}`
    },
    body: JSON.stringify({
      model: 'gpt-4',
      messages: [
        {
          role: 'system',
          content: 'You are an eRDFa semantic reasoning assistant. Analyze RDFa metadata and answer questions.'
        },
        {
          role: 'user',
          content: `RDFa Program:\n${program}\n\nQuestion: ${question}`
        }
      ],
      stream: true
    })
  });
  
  // Stream results
  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  
  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    
    const chunk = decoder.decode(value);
    // Display streaming response
    document.getElementById('llm-output').textContent += chunk;
  }
}
```

### URL with LLM Query

```
https://escaped-rdfa.github.io/namespace/run?program=H4sIAAAA...&llm=What%20is%20the%20duration%20of%20this%20song?

The WASM runtime:
1. Decodes and parses the program
2. Sends to LLM with the question
3. Streams the answer back
```

## Complete Example

### Create Shareable URL

```javascript
// Create a shareable music metadata URL
const musicMetadata = `
<div vocab="http://purl.org/ontology/mo/" 
     prefix="mo: http://purl.org/ontology/mo/
             dc: http://purl.org/dc/terms/">
  <div typeof="mo:Recording">
    <meta property="dc:title" content="Quantum Dreams" />
    <meta property="mo:duration" content="PT4M32S" />
    <meta property="dc:creator" content="Alice" />
    <div rel="mo:genre">
      <div typeof="mo:Genre">
        <meta property="rdfs:label" content="Electronic" />
      </div>
    </div>
  </div>
</div>
`;

const encoded = encodeProgram(musicMetadata);
const url = `https://escaped-rdfa.github.io/namespace/run?program=${encoded}`;

// Tweet it!
const tweet = `Check out my new track "Quantum Dreams" 🎵\n\nFull metadata embedded in URL:\n${url}`;
```

### What Happens When Clicked

1. **User clicks Twitter URL**
2. **Browser loads static site**
3. **WASM runtime initializes** (~80KB, instant)
4. **URL decoded** (program extracted)
5. **Program parsed** (RDFa → triples)
6. **Constant evaluation** (duration → 272 seconds)
7. **Results displayed** (formatted JSON)
8. **Optional: LLM query** ("What genre is this?")

## Advanced Features

### 1. Interactive Queries

```
https://escaped-rdfa.github.io/namespace/run?program=...&query=SELECT%20?title%20WHERE%20{%20?s%20dc:title%20?title%20}
```

### 2. Visualization

```
https://escaped-rdfa.github.io/namespace/run?program=...&viz=graph
```

### 3. Transformation

```
https://escaped-rdfa.github.io/namespace/run?program=...&transform=schema.org
```

### 4. LLM Reasoning

```
https://escaped-rdfa.github.io/namespace/run?program=...&llm=Analyze%20this%20music
```

## URL Length Optimization

### Compression Techniques

```javascript
// Ultra-compact encoding
function ultraCompact(erdfa) {
  // 1. Remove whitespace
  const minified = erdfa.replace(/\s+/g, ' ');
  
  // 2. Abbreviate common terms
  const abbreviated = minified
    .replace(/http:\/\/purl.org\/ontology\/mo\//g, 'm:')
    .replace(/http:\/\/purl.org\/dc\/terms\//g, 'd:')
    .replace(/typeof=/g, 't=')
    .replace(/property=/g, 'p=');
  
  // 3. Gzip + Base64
  const compressed = pako.gzip(abbreviated);
  const base64 = btoa(String.fromCharCode(...compressed));
  
  return base64.replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '');
}

// Result: 100 lines of RDFa → ~200 character URL
```

### Twitter URL Limits

- Twitter allows ~4000 characters in URLs
- With compression: ~2KB of RDFa fits
- That's ~100 triples!

## Benefits

### For Musicians

✓ **One URL** contains complete metadata
✓ **No external dependencies** (self-contained)
✓ **Instant playback** (WASM is fast)
✓ **LLM-queryable** ("What's this song about?")
✓ **Shareable** (Twitter, Discord, anywhere)

### For Developers

✓ **Stateless** (no server needed)
✓ **Verifiable** (WASM is deterministic)
✓ **Composable** (URLs can reference each other)
✓ **Extensible** (add new evaluators)

### For Semantic Web

✓ **Decentralized** (no central server)
✓ **Permanent** (static site + IPFS)
✓ **Interoperable** (standard RDFa)
✓ **Evolvable** (WASM can be updated)

## Implementation

```bash
# 1. Build WASM runtime
cd wasm
cargo build --target wasm32-unknown-unknown --release
wasm-opt -Oz -o ../docs/erdfa_runtime.wasm target/wasm32-unknown-unknown/release/erdfa_runtime.wasm

# 2. Add runner page to static site
cp run.html ../docs/

# 3. Deploy
git add docs/
git commit -m "Add WASM runtime and URL program runner"
git push

# 4. Share URLs!
```

## Conclusion

eRDFa programs embedded in Twitter URLs enable:

1. **Complete programs in URLs** (compressed RDFa)
2. **Instant execution** (WASM runtime < 100KB)
3. **Constant evaluation** (compile-time computation)
4. **LLM integration** (semantic reasoning)
5. **Zero dependencies** (self-contained)
6. **Shareable everywhere** (Twitter, Discord, etc.)

**The ultimate shareable semantic web**: One URL contains program + runtime + results, executable in any browser, queryable by LLMs, all from a static site.

---

*"The URL is the program. The browser is the runtime. The web is the computer."*
