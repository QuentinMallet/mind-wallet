{
  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "nixpkgs/nixos-unstable";
  };

  outputs =
    {
      self,
      fenix,
      flake-utils,
      nixpkgs,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:

      let
        toolchain =
          with fenix.packages.${system};
          combine [
            stable.cargo
            stable.clippy
            stable.rustc
            stable.rustfmt
          ];
        pkgs = nixpkgs.legacyPackages.${system};
        platform = pkgs.makeRustPlatform {
          cargo = toolchain;
          rustc = toolchain;
        };
        lib = pkgs.lib;
        omcSrc =
          if builtins.pathExists ./.omc then
            builtins.path {
              path = ./.omc;
              name = "omc-planning-artifacts";
              filter =
                path: _type:
                let
                  rel = pkgs.lib.removePrefix (toString ./.omc + "/") path;
                in
                pkgs.lib.hasPrefix "plans/" rel
                || pkgs.lib.hasPrefix "specs/" rel
                || pkgs.lib.hasPrefix "research/" rel
                || rel == "";
            }
          else
            null;
        mkGuideDocs =
          {
            includePlanning ? false,
          }:
          pkgs.stdenv.mkDerivation {
            name = "mind-wallet-guide";
            src = ./docs;
            nativeBuildInputs = [ pkgs.mdbook ];
            buildPhase = ''
              cp -r $src build-docs
              chmod -R u+w build-docs
              cd build-docs
              ${pkgs.lib.optionalString (includePlanning && omcSrc != null) ''
                chmod +x scripts/generate-planning-artifacts.sh
                bash scripts/generate-planning-artifacts.sh ${omcSrc} src
              ''}
              mdbook build --dest-dir $out
            '';
            dontInstall = true;
          };
        apiDocs = platform.buildRustPackage {
          pname = "mind-wallet-rustdoc";
          version = "0.1.0";
          dontCheck = true;
          cargoLock.lockFile = ./Cargo.lock;
          src = ./.;
          buildPhase = "cargo doc --offline --no-deps";
          installPhase = ''
            mkdir -p $out
            cp -a target/doc/. $out/
          '';
        };
        mkDoc =
          {
            includePlanning ? false,
          }:
          pkgs.runCommand "mind-wallet-doc" { } ''
            mkdir -p $out/guide $out/api
            cp -r ${mkGuideDocs { inherit includePlanning; }}/. $out/guide/
            cp -r ${apiDocs}/. $out/api/
          '';
        mindWallet = platform.buildRustPackage {
          pname = "mind-wallet";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };

        # Sandbox-friendly WASM build for the portfolio demo. Uses
        # `cargo build` + `wasm-bindgen` + `wasm-opt` directly instead of
        # `wasm-pack` (which auto-downloads wasm-bindgen-cli at build time
        # and therefore cannot run inside a sealed nix derivation).
        #
        # `pkgs.wasm-bindgen-cli` must match the `wasm-bindgen` crate
        # version exactly — pinned to `=0.2.121` in Cargo.toml. Bump both
        # in lockstep with any nixpkgs upgrade.
        wasmToolchain = fenix.packages.${system}.combine [
          fenix.packages.${system}.stable.cargo
          fenix.packages.${system}.stable.rustc
          fenix.packages.${system}.targets.wasm32-unknown-unknown.stable.rust-std
        ];
        wasmPlatform = pkgs.makeRustPlatform {
          cargo = wasmToolchain;
          rustc = wasmToolchain;
        };
        mindWalletWasm = wasmPlatform.buildRustPackage {
          pname = "mind-wallet-wasm";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = with pkgs; [ wasm-bindgen-cli binaryen ];
          cargoBuildFlags = [
            "--lib"
            "--features"
            "wasm"
          ];
          CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
          # The lib has no `[[test]]`s on wasm32 that we want to run inside
          # the build sandbox (the wasm-bindgen-test harness needs node);
          # they are covered by `wasm-pack test --node` in the devshell.
          doCheck = false;
          buildPhase = ''
            runHook preBuild
            cargo build --release \
              --target wasm32-unknown-unknown \
              --features wasm \
              --lib \
              --offline
            runHook postBuild
          '';
          installPhase = ''
            runHook preInstall
            mkdir -p $out/pkg
            wasm-bindgen \
              target/wasm32-unknown-unknown/release/mind_wallet.wasm \
              --out-dir $out/pkg \
              --target web
            wasm-opt -Oz \
              $out/pkg/mind_wallet_bg.wasm \
              -o $out/pkg/mind_wallet_bg.wasm.opt
            mv $out/pkg/mind_wallet_bg.wasm.opt $out/pkg/mind_wallet_bg.wasm
            runHook postInstall
          '';
        };
        cargoFmtCheck = pkgs.runCommand "mind-wallet-cargo-fmt-check" { } ''
          cp -r ${self} source
          chmod -R u+w source
          cd source
          find src tests -name '*.rs' -print0 \
            | xargs -0 ${toolchain}/bin/rustfmt --edition 2024 --check
          touch $out
        '';
        cargoClippyCheck = platform.buildRustPackage {
          pname = "mind-wallet-cargo-clippy";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          buildPhase = "cargo clippy --offline --all-targets --all-features -- -D warnings";
          installPhase = "touch $out";
          doCheck = false;
        };
      in
      {
        packages = {
          mind-wallet = mindWallet;
          mind-wallet-wasm = mindWalletWasm;
          default = mindWallet;
          doc = mkDoc { };
          doc-with-planning = mkDoc { includePlanning = true; };
        };

        apps = {
          mind-wallet = flake-utils.lib.mkApp {
            drv = mindWallet;
          };
          default = self.apps.${system}.mind-wallet;
        };

        checks = {
          mind-wallet = mindWallet;
          mind-wallet-wasm = mindWalletWasm;
          cargo-fmt = cargoFmtCheck;
          cargo-clippy = cargoClippyCheck;
          docs = self.packages.${system}.doc;
        };

        formatter = pkgs.nixfmt;

        legacyPackages = {
          inherit (self.packages.${system}) mind-wallet;
        };

        devShells = {
          default = pkgs.mkShell {
            # Note: `inputsFrom = [ mindWallet ]` is intentionally NOT used.
            # `mindWallet` brings the original 5-component toolchain via its
            # nativeBuildInputs, which would PATH-shadow the wasm-extended
            # toolchain below and cause `wasm-pack build` to error with
            # "wasm32-unknown-unknown target not found in sysroot". Brave
            # devshell users wanting C deps from mindWallet can add them
            # explicitly to buildInputs.
            buildInputs = [
              # Fenix stable toolchain bundled with the wasm32 target so
              # `wasm-pack build --target web` can resolve `core` for
              # `wasm32-unknown-unknown` without a separate `rustup target add`.
              (fenix.packages.${system}.combine [
                fenix.packages.${system}.stable.cargo
                fenix.packages.${system}.stable.clippy
                fenix.packages.${system}.stable.rust-src
                fenix.packages.${system}.stable.rustc
                fenix.packages.${system}.stable.rustfmt
                fenix.packages.${system}.targets.wasm32-unknown-unknown.stable.rust-std
              ])
            ]
            ++ (with pkgs; [
              mdbook
              monero-cli
              nixfmt
              # WASM portfolio demo tooling (Fork A1 / Fork D1).
              wasm-pack
              binaryen # provides `wasm-opt` used by deploy-pages.yml (best-effort)
            ]);
          };
        };
      }
    )
    // {
      overlays.default = _final: prev: {
        mind-wallet = self.packages.${prev.stdenv.hostPlatform.system}.mind-wallet;
      };

      nixosModules.default = self.nixosModules.mind-wallet;
      nixosModules.mind-wallet =
        {
          config,
          lib,
          pkgs,
          ...
        }:
        let
          cfg = config.programs.mind-wallet;
        in
        {
          options.programs.mind-wallet = {
            enable = lib.mkEnableOption "mind-wallet CLI";
            package = lib.mkOption {
              type = lib.types.package;
              default = self.packages.${pkgs.stdenv.hostPlatform.system}.mind-wallet;
              defaultText = lib.literalExpression "inputs.mind-wallet.packages.\${pkgs.stdenv.hostPlatform.system}.mind-wallet";
              description = "The mind-wallet package to install.";
            };
          };

          config = lib.mkIf cfg.enable {
            environment.systemPackages = [ cfg.package ];
          };
        };

      homeManagerModules.default = self.homeManagerModules.mind-wallet;
      homeManagerModules.mind-wallet =
        {
          config,
          lib,
          pkgs,
          ...
        }:
        let
          cfg = config.programs.mind-wallet;
        in
        {
          options.programs.mind-wallet = {
            enable = lib.mkEnableOption "mind-wallet CLI";
            package = lib.mkOption {
              type = lib.types.package;
              default = self.packages.${pkgs.stdenv.hostPlatform.system}.mind-wallet;
              defaultText = lib.literalExpression "inputs.mind-wallet.packages.\${pkgs.stdenv.hostPlatform.system}.mind-wallet";
              description = "The mind-wallet package to install.";
            };
          };

          config = lib.mkIf cfg.enable {
            home.packages = [ cfg.package ];
          };
        };
    };
}
