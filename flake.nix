{
  description = "Leptos Query - A powerful data fetching and caching library for Leptos applications";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    
    # Development tools
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    
    # WASM toolchain
    rust-wasm = {
      url = "github:rustwasm/wasm-pack";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, crane, rust-wasm }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        # Rust toolchain with WASM support
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "wasm32-unknown-unknown" ];
        };
        
        # Node.js and pnpm
        nodejs = pkgs.nodejs_20;
        pnpm = pkgs.nodePackages.pnpm;
        
        # Development tools
        devTools = with pkgs; [
          # Rust tools
          rustToolchain
          rust-analyzer
          cargo-watch
          cargo-tarpaulin
          cargo-audit
          
          # WASM tools
          wasm-pack
          wasm-bindgen-cli
          trunk
          
          # Node.js tools
          nodejs
          pnpm
          
          # System tools
          git
          gcc
          pkg-config
          openssl
          
          # Additional development tools
          jq
          ripgrep
          fd
          bat
          exa
        ];
        
        # Build inputs for the Rust project
        buildInputs = with pkgs; [
          openssl
          pkg-config
        ];
        
        # Runtime dependencies
        runtimeDeps = with pkgs; [
          openssl
        ];
        
      in {
        # Development shell
        devShells.default = pkgs.mkShell {
          buildInputs = devTools;
          
          # Environment variables
          shellHook = ''
            echo "🚀 Welcome to Leptos Query development environment!"
            echo ""
            echo "Available tools:"
            echo "  • Rust: $(rustc --version)"
            echo "  • Cargo: $(cargo --version)"
            echo "  • Node.js: $(node --version)"
            echo "  • pnpm: $(pnpm --version)"
            echo "  • Trunk: $(trunk --version)"
            echo "  • wasm-pack: $(wasm-pack --version)"
            echo ""
            echo "Quick commands:"
            echo "  • cargo test          - Run Rust tests"
            echo "  • cargo bench         - Run benchmarks"
            echo "  • pnpm test:e2e       - Run Playwright tests"
            echo "  • trunk serve         - Serve demo app"
            echo "  • wasm-pack build     - Build WASM"
            echo ""
            
            # Set up Rust environment
            export RUST_BACKTRACE=1
            export RUST_LOG=info
            
            # Set up Node.js environment
            export NODE_ENV=development
            
            # Add local binaries to PATH
            export PATH="$PWD/target/debug:$PATH"
            export PATH="$PWD/demo/node_modules/.bin:$PATH"
          '';
          
          # Rust-specific environment
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "${pkgs.llvmPackages.lld}/bin/ld.lld";
        };
        
        # Build the Rust project
        packages.default = crane.lib.${system}.buildPackage {
          src = ./.;
          cargoArtifacts = crane.lib.${system}.buildDepsOnly {
            inherit src;
            cargoExtraArgs = "--all-features";
          };
          
          buildInputs = buildInputs;
          
          # Build features
          cargoExtraArgs = "--all-features";
          
          # WASM build
          doCheck = true;
          checkPhase = ''
            cargo test --all-features
            cargo bench --all-features
          '';
          
          # Install phase
          installPhase = ''
            mkdir -p $out/bin
            cp target/release/leptos-query-rs $out/bin/
            
            # Copy documentation
            mkdir -p $out/share/doc
            cp -r docs $out/share/doc/
            cp README.md $out/share/doc/
            cp LICENSE $out/share/doc/
          '';
        };
        
        # WASM package
        packages.wasm = pkgs.stdenv.mkDerivation {
          name = "leptos-query-wasm";
          src = ./.;
          
          buildInputs = [ rustToolchain wasm-pack ];
          
          buildPhase = ''
            wasm-pack build --target web --out-dir dist
          '';
          
          installPhase = ''
            mkdir -p $out
            cp -r dist/* $out/
          '';
        };
        
        # Demo app
        packages.demo = pkgs.stdenv.mkDerivation {
          name = "leptos-query-demo";
          src = ./demo;
          
          buildInputs = [ nodejs pnpm trunk ];
          
          buildPhase = ''
            cd demo
            pnpm install
            trunk build
          '';
          
          installPhase = ''
            mkdir -p $out
            cp -r dist/* $out/
          '';
        };
        
        # Development tools
        packages.dev-tools = pkgs.symlinkJoin {
          name = "leptos-query-dev-tools";
          paths = devTools;
        };
        
        # Apps
        apps = {
          # Development server
          dev = {
            type = "app";
            program = toString (pkgs.writeShellScript "dev" ''
              cd demo
              trunk serve
            '');
          };
          
          # Test runner
          test = {
            type = "app";
            program = toString (pkgs.writeShellScript "test" ''
              cargo test --all-features
              cd demo && pnpm test:e2e
            '');
          };
          
          # Benchmarks
          bench = {
            type = "app";
            program = toString (pkgs.writeShellScript "bench" ''
              cargo bench --all-features
            '');
          };
          
          # WASM build
          wasm = {
            type = "app";
            program = toString (pkgs.writeShellScript "wasm" ''
              wasm-pack build --target web --out-dir dist
            '');
          };
        };
        
        # Checks
        checks = {
          # Rust tests
          rust-tests = crane.lib.${system}.cargoTest {
            src = ./.;
            cargoExtraArgs = "--all-features";
          };
          
          # Rust clippy
          rust-clippy = crane.lib.${system}.cargoClippy {
            src = ./.;
            cargoExtraArgs = "--all-features";
            clippyExtraArgs = "-- -D warnings";
          };
          
          # Rust audit
          rust-audit = crane.lib.${system}.cargoAudit {
            src = ./.;
          };
          
          # Format check
          rust-fmt = crane.lib.${system}.cargoFmt {
            src = ./.;
          };
        };
      }
    );
}
