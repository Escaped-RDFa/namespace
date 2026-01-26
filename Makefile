.PHONY: all prove wasm site clean

all: prove wasm site

# Run MiniZinc proofs
prove:
	@echo "🔬 Running optimal shard distribution..."
	@mkdir -p _site/proofs
	@./scripts/solve_shards.sh
	@echo "✓ Proofs complete"

# Build WASM runtime
wasm:
	@echo "🦀 Building WASM runtime..."
	cd wasm && wasm-pack build --target web --out-dir ../docs/wasm
	wasm-opt -Oz docs/wasm/erdfa_runtime_bg.wasm -o docs/wasm/erdfa_runtime.wasm
	@echo "✓ WASM built ($(shell du -h docs/wasm/erdfa_runtime.wasm | cut -f1))"

# Build GitHub Pages site
site: prove wasm
	@echo "📚 Building documentation site..."
	mkdir -p _site/proofs
	mkdir -p _site/wasm
	
	# Copy specs
	cp -r spec _site/
	
	# Copy docs
	cp -r docs _site/
	
	# Copy WASM runtime
	cp -r docs/wasm _site/
	
	# Copy proofs
	cp -r _site/proofs docs/
	
	# Generate index
	./scripts/generate_index.sh > _site/index.html
	
	@echo "✓ Site built in _site/"

# Clean build artifacts
clean:
	rm -rf _site/
	rm -rf docs/wasm/
	rm -rf target/
	rm -rf wasm/target/

# Test WASM runtime
test-wasm: wasm
	@echo "🧪 Testing WASM runtime..."
	node scripts/test_wasm.js

# Verify proofs
verify-proofs: prove
	@echo "✅ Verifying MiniZinc proofs..."
	python3 scripts/verify_proofs.py
