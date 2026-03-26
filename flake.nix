# Nix flake for beads_rust - Agent-first issue tracker
#
# Usage:
#   nix build              Build the br binary
#   nix run                Run br directly
#   nix develop            Enter development shell
#   nix flake check        Run all checks (build, clippy, fmt, tests)
#
# First time setup:
#   nix flake lock         Generate flake.lock (commit this file)
#
# The flake uses:
#   - crane: Incremental Rust builds with dependency caching
#   - fenix: Stable Rust toolchain (edition 2024 is stable since 1.85)
#   - flake-utils: Multi-system support
#
{
  description = "beads_rust - Agent-first issue tracker (SQLite + JSONL)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    crane.url = "github:ipetkov/crane";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";

    # Sibling dependencies fetched from GitHub since Nix flakes
    # cannot use relative path dependencies
    toon_rust = {
      url = "github:Dicklesworthstone/toon_rust";
      flake = false;
    };

    frankensqlite = {
      url = "github:Dicklesworthstone/frankensqlite";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, crane, fenix, flake-utils, toon_rust, frankensqlite, ... }:
    flake-utils.lib.eachSystem [
      "x86_64-linux"
      "aarch64-linux"
      "x86_64-darwin"
      "aarch64-darwin"
    ] (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        isLinux = pkgs.stdenv.isLinux;

        # For static musl builds on Linux
        muslTarget = {
          "x86_64-linux" = "x86_64-unknown-linux-musl";
          "aarch64-linux" = "aarch64-unknown-linux-musl";
        }.${system} or null;

        # Cross-compilation pkgs for musl target (provides static C libs)
        muslPkgs = if isLinux then pkgs.pkgsCross.musl64 else null;

        # Stable Rust toolchain via fenix (edition 2024 is stable since 1.85)
        fenixPkgs = fenix.packages.${system};
        rustToolchain = fenixPkgs.combine ([
          fenixPkgs.stable.cargo
          fenixPkgs.stable.rustc
          fenixPkgs.stable.rust-src
          fenixPkgs.stable.clippy
          fenixPkgs.stable.rustfmt
        ] ++ pkgs.lib.optionals (muslTarget != null) [
          fenixPkgs.targets.${muslTarget}.stable.rust-std
        ]);

        # Use host pkgs for crane (build scripts run on host)
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Filter source to include only what's needed for the build
        sourceFilter = path: type:
          (craneLib.filterCargoSources path type)
          || builtins.match ".*\\.toml$" path != null
          || builtins.match ".*\\.rs$" path != null
          || builtins.match ".*\\.sql$" path != null;

        # Combined source tree with beads_rust and sibling dependencies
        # Required because Cargo.toml uses path = "../toon_rust" and "../frankensqlite"
        combinedSrc = pkgs.runCommand "beads_rust-src" { } ''
          mkdir -p $out/beads_rust $out/toon_rust $out/frankensqlite

          # Copy beads_rust
          cp ${./Cargo.toml} $out/beads_rust/Cargo.toml
          cp ${./Cargo.lock} $out/beads_rust/Cargo.lock
          cp ${./build.rs} $out/beads_rust/build.rs
          cp -r ${./src} $out/beads_rust/src

          # Optional directories
          ${pkgs.lib.optionalString (builtins.pathExists ./benches) "cp -r ${./benches} $out/beads_rust/benches"}
          ${pkgs.lib.optionalString (builtins.pathExists ./tests) "cp -r ${./tests} $out/beads_rust/tests"}

          # Copy sibling dependencies
          cp -r ${toon_rust}/* $out/toon_rust/
          cp -r ${frankensqlite}/* $out/frankensqlite/
        '';

        # Vendor dependencies using the local Cargo.lock directly,
        # since combinedSrc nests it under beads_rust/ where crane can't find it
        cargoVendorDir = craneLib.vendorCargoDeps { cargoLock = ./Cargo.lock; };

        # Common arguments shared between dependency and final builds
        commonArgs = {
          src = combinedSrc;
          inherit cargoVendorDir;
          cargoLock = ./Cargo.lock;

          pname = "beads_rust";
          version = "0.1.34";

          strictDeps = true;

          # Build from the beads_rust subdirectory
          postUnpack = ''
            cd $sourceRoot/beads_rust
            sourceRoot=$PWD
          '';

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs =
            if isLinux then [
              muslPkgs.openssl.dev
              muslPkgs.openssl.out
            ] else with pkgs; [
              openssl
              darwin.apple_sdk.frameworks.Security
              darwin.apple_sdk.frameworks.SystemConfiguration
              darwin.apple_sdk.frameworks.CoreFoundation
              libiconv
            ];

          # OpenSSL configuration
          OPENSSL_NO_VENDOR = "1";
          OPENSSL_STATIC = if isLinux then "1" else "";
        } // pkgs.lib.optionalAttrs isLinux {
          # Static musl build on Linux
          CARGO_BUILD_TARGET = muslTarget;
          CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static -C linker=${muslPkgs.stdenv.cc}/bin/${muslPkgs.stdenv.cc.targetPrefix}cc";
          # Point pkg-config at musl OpenSSL
          PKG_CONFIG_PATH = "${muslPkgs.openssl.dev}/lib/pkgconfig";
        };

        # Build only dependencies (cached between builds)
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Full package build
        beads_rust = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;

          doCheck = false;  # Tests run separately in checks

          meta = with pkgs.lib; {
            description = "Agent-first issue tracker (SQLite + JSONL)";
            homepage = "https://github.com/Dicklesworthstone/beads_rust";
            license = licenses.mit;
            mainProgram = "br";
            platforms = platforms.unix;
          };
        });

      in
      {
        # nix build / nix build .#beads_rust
        packages = {
          default = beads_rust;
          inherit beads_rust;
        };

        # nix develop
        devShells.default = craneLib.devShell {
          inputsFrom = [ beads_rust ];

          packages = with pkgs; [
            # Rust tooling
            rust-analyzer
            cargo-watch
            cargo-edit
            cargo-outdated
            cargo-audit
            cargo-expand

            # SQLite
            sqlite

            # TOML
            taplo

            # Testing
            cargo-nextest
            cargo-tarpaulin

            # Performance
            hyperfine
          ];

          shellHook = ''
            export RUST_BACKTRACE=1
            export RUST_LOG=info
            echo "beads_rust dev shell - Rust $(rustc --version | cut -d' ' -f2)"
          '';
        };

        # nix flake check
        checks = {
          inherit beads_rust;

          clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          fmt = craneLib.cargoFmt {
            src = combinedSrc;
            inherit cargoVendorDir;
            postUnpack = ''
              cd $sourceRoot/beads_rust
              sourceRoot=$PWD
            '';
          };

          tests = craneLib.cargoTest (commonArgs // {
            inherit cargoArtifacts;
          });
        };

        # nix run
        apps.default = flake-utils.lib.mkApp {
          drv = beads_rust;
          name = "br";
        };

        # For use as overlay in other flakes
        overlays.default = final: prev: {
          beads_rust = beads_rust;
          br = beads_rust;
        };
      });
}
