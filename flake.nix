{
  description = "Adunata, a web application for choirs and instrumental ensembles to organize gatherings";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    crane.url = "github:ipetkov/crane/v0.24.0";
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});

      build =
        pkgs:
        let
          craneLib = crane.mkLib pkgs;
          npmDeps = pkgs.importNpmLock { npmRoot = ./client; };

          clientPackage =
            args:
            pkgs.buildNpmPackage (
              {
                pname = "adunata-client";
                version = "0.1.0";
                src = ./client;
                inherit npmDeps;
                npmConfigHook = pkgs.importNpmLock.npmConfigHook;
              }
              // args
            );

          client = clientPackage {
            installPhase = "cp -r dist $out";
          };

          serverSource = craneLib.cleanCargoSource ./server;
          serverArgs = {
            src = serverSource;
            strictDeps = true;
            preBuild = ''
              mkdir -p ../client
              ln -sfn ${client} ../client/dist
            '';
          };
          cargoArtifacts = craneLib.buildDepsOnly serverArgs;
        in
        {
          inherit client;

          server = craneLib.buildPackage (serverArgs // { inherit cargoArtifacts; });

          clippy = craneLib.cargoClippy (
            serverArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          rustFormatting = craneLib.cargoFmt { src = serverSource; };

          clientTest = clientPackage {
            pname = "adunata-client-test";
            buildPhase = "npm test";
            installPhase = "touch $out";
          };

          clientLint = clientPackage {
            pname = "adunata-client-lint";
            buildPhase = "npm run lint -- --deny-warnings";
            installPhase = "touch $out";
          };
        };
    in
    {
      formatter = forAllSystems (pkgs: pkgs.nixfmt);

      packages = forAllSystems (pkgs: {
        default = (build pkgs).server;
      });

      checks = forAllSystems (
        pkgs:
        let
          built = build pkgs;
        in
        {
          inherit (built)
            server
            clippy
            rustFormatting
            clientLint
            clientTest
            ;

          nixFormatting = pkgs.runCommand "check-nix-formatting" { nativeBuildInputs = [ pkgs.nixfmt ]; } ''
            find ${self} -name '*.nix' -exec nixfmt --check {} +
            touch $out
          '';

          editorconfig =
            pkgs.runCommand "check-editorconfig" { nativeBuildInputs = [ pkgs.editorconfig-checker ]; }
              ''
                cd ${self}
                editorconfig-checker
                touch $out
              '';
        }
      );

      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          packages = [
            pkgs.cargo
            pkgs.rustc
            pkgs.clippy
            pkgs.rustfmt
            pkgs.rust-analyzer
            pkgs.nodejs_24
            pkgs.postgresql_17
            pkgs.mailpit
          ];

          PGHOST = "localhost";
          PGPORT = "5432";
          MP_SMTP_BIND_ADDR = "localhost:1025";
          MP_UI_BIND_ADDR = "localhost:8025";
        };
      });
    };
}
