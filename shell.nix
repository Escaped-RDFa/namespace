{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # GitHub Actions testing
    act
    docker  # act needs docker
    
    # Build tools (from original shell.nix)
    minizinc
    gecode
    rustc
    cargo
    wasm-pack
    wasm-bindgen-cli
    binaryen
    brotli
    gzip
    mdbook
    nodejs
    python3
  ];
  
  shellHook = ''
    echo "🔐 eRDFa Development Environment with act2nix"
    echo "=============================================="
    echo "act: $(act --version 2>/dev/null || echo 'not found')"
    echo ""
    echo "Available commands:"
    echo "  make prove                  - Run MiniZinc proofs"
    echo "  make wasm                   - Build WASM runtime"
    echo "  make all                    - Build everything"
    echo "  act -j build                - Test GitHub Actions locally"
  '';
}
