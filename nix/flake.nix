{
  description = "Eco-Bot: Ecological & Economic Code Analysis Platform";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    # Haskell
    haskell-flake.url = "github:srid/haskell-flake";

    # OCaml
    opam-nix.url = "github:tweag/opam-nix";

    # Rust
    rust-overlay.url = "github:oxalica/rust-overlay";

    # Deno
    deno2nix.url = "github:SnO2WMaN/deno2nix";
  };

  outputs = { self, nixpkgs, flake-utils, haskell-flake, opam-nix, rust-overlay, deno2nix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };

        # Haskell packages
        haskellPackages = pkgs.haskellPackages.extend (hself: hsuper: {
          eco-analyzer = hself.callCabal2nix "eco-analyzer"
            ./analyzers/code-haskell { };
        });

        # OCaml packages via opam-nix
        ocamlScope = opam-nix.lib.${system}.buildOpamProject { } "eco-doc-analyzer" ./analyzers/docs-ocaml { };

        # Common development tools
        commonDeps = with pkgs; [
          # Version control
          git
          git-lfs

          # Build tools
          gnumake
          cmake
          pkg-config

          # JSON/YAML processing
          jq
          yq

          # Search/navigation
          ripgrep
          fd
          bat
          fzf

          # Container tools
          nerdctl
          containerd
          buildkit

          # Databases
          arangodb
          # virtuoso-opensource  # If available

          # Logic programming
          souffle
          swiProlog
        ];

        # Haskell development environment
        haskellDeps = with pkgs; [
          ghc
          cabal-install
          haskell-language-server
          hlint
          ormolu
          haskellPackages.ghcid
        ];

        # OCaml development environment
        ocamlDeps = with pkgs; [
          ocaml
          dune_3
          opam
          ocamlPackages.merlin
          ocamlPackages.ocaml-lsp
          ocamlPackages.ocamlformat
          ocamlPackages.utop
          ocamlPackages.yojson
          ocamlPackages.ppx_deriving
          ocamlPackages.re
          ocamlPackages.omd
          ocamlPackages.cmdliner
        ];

        # ReScript + Deno
        rescriptDeps = with pkgs; [
          deno
          nodejs_20  # Only for ReScript compiler
        ];

        # Python for policy engine
        pythonDeps = with pkgs; [
          (python311.withPackages (ps: with ps; [
            numpy
            scipy
            torch
            networkx
            pyyaml
            aiohttp
            pytest
            black
            mypy
            ruff
          ]))
        ];

        # Rust for orchestrator
        rustDeps = with pkgs; [
          (rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" "rust-analyzer" ];
          })
        ];

      in
      {
        # Development shell
        devShells.default = pkgs.mkShell {
          buildInputs = commonDeps ++ haskellDeps ++ ocamlDeps ++ rescriptDeps ++ pythonDeps ++ rustDeps;

          shellHook = ''
            echo "🌱 Eco-Bot Development Environment"
            echo "=================================="
            echo ""
            echo "Languages:"
            echo "  - Haskell: $(ghc --version | head -1)"
            echo "  - OCaml:   $(ocaml --version)"
            echo "  - Deno:    $(deno --version | head -1)"
            echo "  - Python:  $(python --version)"
            echo "  - Rust:    $(rustc --version)"
            echo ""
            echo "Available commands:"
            echo "  make build     - Build all components"
            echo "  make test      - Run all tests"
            echo "  make container - Build container image"
            echo ""

            # Set up environment
            export DENO_DIR="$PWD/.deno"
            export CABAL_DIR="$PWD/.cabal"

            # Python virtual environment
            if [ ! -d ".venv" ]; then
              python -m venv .venv
            fi
            source .venv/bin/activate
          '';
        };

        # Haskell-only shell
        devShells.haskell = pkgs.mkShell {
          buildInputs = commonDeps ++ haskellDeps;
        };

        # OCaml-only shell
        devShells.ocaml = pkgs.mkShell {
          buildInputs = commonDeps ++ ocamlDeps;
        };

        # Bot integration shell (ReScript + Deno)
        devShells.bot = pkgs.mkShell {
          buildInputs = commonDeps ++ rescriptDeps;
        };

        # Policy engine shell (Python)
        devShells.policy = pkgs.mkShell {
          buildInputs = commonDeps ++ pythonDeps;
        };

        # Packages
        packages = {
          eco-analyzer = haskellPackages.eco-analyzer;

          # Container image
          container = pkgs.dockerTools.buildLayeredImage {
            name = "eco-bot";
            tag = "latest";

            contents = with pkgs; [
              haskellPackages.eco-analyzer
              deno
              souffle
              swiProlog
              cacert
              coreutils
              bash
            ];

            config = {
              Cmd = [ "deno" "run" "--allow-net" "--allow-env" "--allow-read" "/app/bot-integration/src/Main.res.js" ];
              WorkingDir = "/app";
              Env = [
                "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
              ];
              ExposedPorts = {
                "3000/tcp" = {};
              };
            };
          };

          default = self.packages.${system}.container;
        };

        # Apps
        apps = {
          eco-bot = {
            type = "app";
            program = "${pkgs.deno}/bin/deno run --allow-net --allow-env --allow-read bot-integration/src/Main.res.js";
          };

          default = self.apps.${system}.eco-bot;
        };
      }
    );
}
