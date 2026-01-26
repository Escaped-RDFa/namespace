# Lean4 Introspection Plugin

**Status**: Example plugin demonstrating universal ontology pattern

## The Pattern (Size >= 71)

1. **Lean4 Dump** - JSON from introspector
2. **Lean4 Dumper** - Tool that generated it
3. **Lean4 Dumper Trace** - Provenance chain
4. **addr2line** - Source location mapping
5. **Source Code** - Original Lean4 code
6. **Verification** - JSON ≡ Source Code
7. **Universal Ontology** - Maps to 71+ shards

## Example

```rust
UniversalOntology {
    size: 71,  // Gandalf threshold
    dump: Lean4Dump { ... },
    trace: Lean4Trace {
        dumper: "lean4-introspector",
        addr: "0x686e510a",
        source: "https://huggingface.co/..."
    },
    verified: true
}
```

## Dataset

Source: [introspector/MicroLean4](https://huggingface.co/datasets/introspector/MicroLean4)

Example: [SimpleExpr.rec_686e510a...](https://huggingface.co/datasets/introspector/MicroLean4/raw/main/SimpleExpr.rec_686e510a6699f2e1ff1b216c16d94cd379ebeca00c030a79a3134adff699e06c.json)

## Plugin Architecture

Lean4 is **not core** - it's an example showing how any language/system can be:
1. Dumped to JSON
2. Traced to source
3. Verified
4. Mapped to universal ontology (71+ shards)

Other plugins: Python AST, Rust HIR, Coq, Agda, etc.
