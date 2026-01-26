#!/usr/bin/env python3
"""Import Lean4 introspection JSON into eRDFa shards"""

import json
import sys

def lean4_to_erdfa(lean4_json):
    """Convert Lean4 SimpleExpr to eRDFa triple"""
    return {
        "@context": "http://escaped-rdfa.org/ns#",
        "@type": "Lean4Expr",
        "kind": lean4_json.get("kind", "unknown"),
        "name": lean4_json.get("cnstInfB", {}).get("name", ""),
        "proof": True
    }

def main():
    # Hugging Face dataset URL
    hf_url = "https://huggingface.co/datasets/introspector/MicroLean4/raw/main/SimpleExpr.rec_686e510a6699f2e1ff1b216c16d94cd379ebeca00c030a79a3134adff699e06c.json"
    
    input_file = sys.argv[1] if len(sys.argv) > 1 else "/mnt/data1/nix/time/2025/06/01/solfunmeme-dioxus/hf_dataset/reports/src_model_lean/simple_expr_type.rs.json"
    
    with open(input_file) as f:
        data = json.load(f)
    
    erdfa = lean4_to_erdfa(data)
    print(json.dumps(erdfa, indent=2))

if __name__ == "__main__":
    main()
