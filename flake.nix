{
  description = "Adunata, a web application for choirs and instrumental ensembles to organize gatherings";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    crane.url = "github:ipetkov/crane/v0.24.0";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      fenix,
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
                src = pkgs.lib.fileset.toSource {
                  root = ./.;
                  fileset = pkgs.lib.fileset.intersection (pkgs.lib.fileset.gitTracked ./.) (
                    pkgs.lib.fileset.unions [
                      ./client
                      ./server/locales
                    ]
                  );
                };
                sourceRoot = "source/client";
                inherit npmDeps;
                npmConfigHook = pkgs.importNpmLock.npmConfigHook;
              }
              // args
            );

          client = clientPackage {
            installPhase = "cp -r dist $out";
          };

          serverSource = pkgs.lib.fileset.toSource {
            root = ./server;
            fileset = pkgs.lib.fileset.unions [
              (craneLib.fileset.commonCargoSources ./server)
              ./server/migrations
              ./server/locales
            ];
          };
          serverArgs = {
            src = serverSource;
            strictDeps = true;
            meta.mainProgram = "adunata";
            preBuild = ''
              mkdir -p ../client
              ln -sfn ${client} ../client/dist
            '';
          };
          testServices = {
            # The tests bind PostgreSQL and Mailpit to the loopback address, which the macOS sandbox forbids otherwise.
            __darwinAllowLocalNetworking = true;
            nativeCheckInputs = [
              pkgs.postgresql_17
              pkgs.mailpit
            ];
            preCheck = ''
              export PGDATA="$TMPDIR/pgdata" PGHOST="$TMPDIR/pgsocket" PGUSER=adunata PGDATABASE=postgres
              mkdir -p "$PGHOST"
              initdb --username="$PGUSER" --auth=trust --no-instructions >/dev/null
              pg_ctl start --wait --silent --options="-k $PGHOST -c listen_addresses="
              export ADNT_DATABASE_URL="postgres:///postgres?host=$PGHOST&user=$PGUSER"
              export ADNT_SMTP_PORT=11025 MAILPIT_UI_PORT=18025
              mailpit --smtp "127.0.0.1:$ADNT_SMTP_PORT" --listen "127.0.0.1:$MAILPIT_UI_PORT" --quiet &
              mailpitPid=$!
              for _ in $(seq 50); do (echo > "/dev/tcp/127.0.0.1/$MAILPIT_UI_PORT") 2>/dev/null && break; sleep 0.1; done
              stopTestServices() {
                kill "$mailpitPid"
                pg_ctl stop --wait --silent
              }
              failureHooks+=(stopTestServices)
            '';
            postCheck = "stopTestServices";
          };
          # Dependencies compile without the client, so a change to the client keeps them cached.
          forDependencies = args: removeAttrs args [ "preBuild" ];
          cargoArtifacts = craneLib.buildDepsOnly (forDependencies serverArgs);
          serverArtifacts = craneLib.cargoBuild (
            serverArgs
            // {
              inherit cargoArtifacts;
              doCheck = false;
            }
          );
          server = craneLib.buildPackage (
            serverArgs
            // {
              cargoArtifacts = serverArtifacts;
              doCheck = false;
            }
          );
          testArtifacts = craneLib.mkCargoDerivation (
            serverArgs
            // {
              cargoArtifacts = serverArtifacts;
              pnameSuffix = "-tests";
              buildPhaseCargoCommand = "cargoWithProfile test --locked --bins --test startup --no-run";
            }
          );
          serverTests =
            args: suffix: command:
            craneLib.mkCargoDerivation (
              args
              // {
                cargoArtifacts = testArtifacts;
                pnameSuffix = suffix;
                buildPhaseCargoCommand = "";
                doCheck = true;
                checkPhaseCargoCommand = command;
                doInstallCargoArtifacts = false;
              }
            );
          openapi = pkgs.runCommand "adunata-openapi" { } "${server}/bin/adunata openapi > $out";

          muslTargets = {
            x86_64-linux = "x86_64-unknown-linux-musl";
            aarch64-linux = "aarch64-unknown-linux-musl";
          };
          system = pkgs.stdenv.hostPlatform.system;

          # A statically linked binary that runs on any Linux distribution. The release archive and the image both carry it.
          staticServer =
            let
              target = muslTargets.${system};
              targetVariable = pkgs.lib.toUpper (builtins.replaceStrings [ "-" ] [ "_" ] target);
              staticCc = pkgs.pkgsStatic.stdenv.cc;
              toolchain = fenix.packages.${system}.combine [
                fenix.packages.${system}.stable.cargo
                fenix.packages.${system}.stable.rustc
                fenix.packages.${system}.targets.${target}.stable.rust-std
              ];
              muslCrane = (crane.mkLib pkgs).overrideToolchain toolchain;
              args = serverArgs // {
                CARGO_BUILD_TARGET = target;
                "CARGO_TARGET_${targetVariable}_LINKER" = "${staticCc}/bin/${staticCc.targetPrefix}cc";
                "CC_${builtins.replaceStrings [ "-" ] [ "_" ] target}" =
                  "${staticCc}/bin/${staticCc.targetPrefix}cc";
                HOST_CC = "${pkgs.stdenv.cc}/bin/cc";
                doCheck = false;
              };
            in
            muslCrane.buildPackage (
              args // { cargoArtifacts = muslCrane.buildDepsOnly (forDependencies args); }
            );

          release = pkgs.runCommand "adunata-release-${muslTargets.${system}}" { } ''
            mkdir $out
            cp ${staticServer}/bin/adunata $out/adunata
            cp ${./deploy/systemd/adunata.service} $out/adunata.service
            cp ${./deploy/systemd/adunata.env.example} $out/adunata.env.example
            cp ${./LICENSE} $out/LICENSE
          '';

          image = pkgs.dockerTools.buildLayeredImage {
            name = "adunata";
            tag = "latest";
            config = {
              Cmd = [
                "adunata"
                "serve"
              ];
              Env = [
                "ADNT_HTTP_LISTEN_ADDRESS=0.0.0.0:8080"
                "PATH=${staticServer}/bin"
              ];
              ExposedPorts."8080/tcp" = { };
              User = "65534:65534";
            };
          };

          # Starts PostgreSQL, Mailpit and the release binary on port 18080 with the organizer organizer@example.org and leaves the binary's PID in $adunataPid.
          startInstance = ''
            export PGDATA="$TMPDIR/pgdata" PGHOST="$TMPDIR/pgsocket" PGUSER=adunata
            mkdir -p "$PGHOST"
            initdb --username="$PGUSER" --auth=trust --no-instructions >/dev/null
            pg_ctl start --wait --silent --options="-k $PGHOST -c listen_addresses="
            mailpit --smtp 127.0.0.1:11025 --listen 127.0.0.1:18025 --quiet &
            ADNT_DATABASE_URL="postgres:///postgres?host=$PGHOST&user=$PGUSER" \
              ADNT_HTTP_LISTEN_ADDRESS=127.0.0.1:18080 ADNT_HTTP_PUBLIC_URL=http://localhost:18080 \
              ADNT_SMTP_HOST=127.0.0.1 ADNT_SMTP_PORT=11025 ADNT_SMTP_ENCRYPTION=none \
              ADNT_MAIL_SENDER="Adunata <adunata@example.org>" ADNT_LOCALIZATION_TIME_ZONE=Europe/Berlin \
              ADNT_BOOTSTRAP_ORGANIZER_MAIL=organizer@example.org ADNT_BOOTSTRAP_ORGANIZER_FIRST_NAME=Ada \
              ADNT_BOOTSTRAP_ORGANIZER_LAST_NAME=Lovelace \
              ${staticServer}/bin/adunata serve &
            adunataPid=$!
            for _ in $(seq 100); do (echo > /dev/tcp/127.0.0.1/18080) 2>/dev/null && break; sleep 0.1; done
          '';
        in
        {
          inherit
            client
            server
            release
            image
            ;

          clippy = craneLib.cargoClippy (
            serverArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          rustFormatting = craneLib.cargoFmt { src = serverSource; };

          serverUnitTests =
            serverTests serverArgs "-unit-tests"
              "cargoWithProfile test --locked --bins -- --skip integration::";

          serverIntegrationTests = serverTests (serverArgs // testServices) "-integration-tests" ''
            cargoWithProfile test --locked --bins -- integration::
            ADUNATA_TEST_BINARY=${server}/bin/adunata cargoWithProfile test --locked --test startup
          '';

          contract = clientPackage {
            pname = "adunata-contract";
            buildPhase = ''
              if ! diff -u openapi.json ${openapi}; then
                echo "client/openapi.json differs from the server. Run: cargo run -- openapi > ../client/openapi.json"
                exit 1
              fi
              cp -r src/api/generated "$TMPDIR/committed"
              npx orval
              if ! diff -ru "$TMPDIR/committed" src/api/generated; then
                echo "Generated client code is out of date. Run: npm run generate"
                exit 1
              fi
            '';
            installPhase = "touch $out";
          };

          clientUnitTests = clientPackage {
            pname = "adunata-client-unit-tests";
            buildPhase = "npm test";
            installPhase = "touch $out";
          };

          clientIntegrationTests = clientPackage {
            pname = "adunata-client-integration-tests";
            buildPhase = "npm run test:integration";
            installPhase = "touch $out";
          };

          memory =
            pkgs.runCommand "check-memory"
              {
                nativeBuildInputs = [
                  pkgs.postgresql_17
                  pkgs.mailpit
                  pkgs.oha
                  pkgs.curl
                ];
                SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
              }
              ''
                ${startInstance}
                api=http://127.0.0.1:18080/api

                # Prints the secret of the newest link of a kind in Mailpit, waiting until its mail arrives.
                newest_secret() {
                  for _ in $(seq 300); do
                    id=$(curl -s 'http://127.0.0.1:18025/api/v1/messages?limit=1' | grep -o '"ID":"[^"]*"' | head -1 | cut -d'"' -f4 || true)
                    found=$(curl -s "http://127.0.0.1:18025/api/v1/message/$id" | grep -o "/$1#[0-9a-f]\{64\}" | head -1 || true)
                    if [ -n "$found" ]; then echo "''${found#*#}"; return; fi
                    sleep 0.1
                  done
                  echo "no $1 link arrived" >&2
                  exit 1
                }

                curl -sf -H 'content-type: application/json' -d '{"mail":"organizer@example.org"}' "$api/sign-in-requests"
                signIn=$(newest_secret sign-in)
                cookie=$(curl -sf -D - -o /dev/null -X POST -H "Authorization: Bearer $signIn" "$api/sessions" \
                  | grep -i '^set-cookie:' | sed 's/^[^:]*: //; s/;.*//')
                gathering=$(curl -sf -H "Cookie: $cookie" -H 'content-type: application/json' \
                  -d "{\"title\":\"Load\",\"location\":\"Stage\",\"firstDay\":\"$(date -d '+30 days' +%F)\",\"lastDay\":\"$(date -d '+32 days' +%F)\"}" \
                  "$api/gatherings" | grep -o '"id":[0-9]*' | head -1 | cut -d: -f2)
                curl -sf -H "Cookie: $cookie" -H 'content-type: application/json' -d '{}' "$api/gatherings/$gathering/invitations"
                secret=$(newest_secret invitation)

                oha --no-tui -z 30s -c 4 -m PUT \
                  -H "Authorization: Bearer $secret" -H 'content-type: application/json' \
                  -d '{"status":"accepted","beds":"offer","bedCount":2,"comment":"Load"}' \
                  "$api/invitation/response" > put.txt &
                putPid=$!
                oha --no-tui -z 30s -c 3 -H "Authorization: Bearer $secret" "$api/invitation" > page.txt &
                pagePid=$!
                oha --no-tui -z 30s -c 3 http://127.0.0.1:18080/invitation > client.txt
                wait "$putPid" "$pagePid"

                peak=$(awk '/VmHWM/ { print $2 }' "/proc/$adunataPid/status")
                echo "Peak resident memory: $((peak / 1024)) MB"
                grep -E 'Requests/sec|Success rate' put.txt page.txt client.txt
                grep -q '\[200\]' put.txt
                grep -q '\[200\]' page.txt
                kill "$adunataPid"
                pg_ctl stop --wait --silent
                test "$peak" -lt $((150 * 1024))
                echo "$peak" > $out
              '';

          endToEnd = clientPackage {
            pname = "adunata-end-to-end";
            nativeBuildInputs = [
              pkgs.postgresql_17
              pkgs.mailpit
            ];
            PLAYWRIGHT_BROWSERS_PATH = pkgs.playwright-driver.browsers;
            PLAYWRIGHT_SKIP_VALIDATE_HOST_REQUIREMENTS = "true";
            FONTCONFIG_FILE = pkgs.makeFontsConf { fontDirectories = [ pkgs.dejavu_fonts ]; };
            ADUNATA_E2E_URL = "http://localhost:18080";
            ADUNATA_E2E_MAILPIT_URL = "http://127.0.0.1:18025";
            buildPhase = ''
              export HOME="$TMPDIR"
              ${startInstance}
              if ! npx playwright test; then
                find test-results -name '*.md' -exec cat {} +
                exit 1
              fi
              kill "$adunataPid"
              pg_ctl stop --wait --silent
            '';
            installPhase = "touch $out";
          };

          clientLint = clientPackage {
            pname = "adunata-client-lint";
            buildPhase = "npm run lint -- --deny-warnings";
            installPhase = "touch $out";
          };
        };

      vmTest =
        pkgs: built:
        import ./nix/test.nix {
          inherit pkgs;
          inherit (built) release image;
          module = self.nixosModules.default;
        };
    in
    {
      formatter = forAllSystems (pkgs: pkgs.nixfmt);

      packages = forAllSystems (
        pkgs:
        let
          built = build pkgs;
        in
        {
          default = built.server;
        }
        // pkgs.lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux { inherit (built) release image; }
      );

      nixosModules.default = import ./nix/module.nix (pkgs: (build pkgs).server);

      checks = forAllSystems (
        pkgs:
        let
          built = build pkgs;
        in
        let
          lintChecks = {
            inherit (built) clippy rustFormatting clientLint;

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
          };
          buildChecks = { inherit (built) server; };
          testChecks = { inherit (built) serverUnitTests clientUnitTests; };
          integrationChecks = { inherit (built) serverIntegrationTests clientIntegrationTests contract; };

          lint = pkgs.linkFarm "lint" lintChecks;
          build = pkgs.linkFarm "build" ({ inherit lint; } // buildChecks);
          test = pkgs.linkFarm "test" ({ inherit build; } // testChecks);
          integration = pkgs.linkFarm "integration" ({ inherit test; } // integrationChecks);
        in
        lintChecks
        // buildChecks
        // testChecks
        // integrationChecks
        // {
          inherit
            lint
            build
            test
            integration
            ;
        }
        // pkgs.lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
          e2e = pkgs.linkFarm "e2e" {
            inherit integration;
            inherit (built) endToEnd;
          };
          memory = pkgs.linkFarm "memory" {
            inherit integration;
            inherit (built) memory;
          };
          nixos = pkgs.linkFarm "nixos" {
            inherit integration;
            vm = vmTest pkgs built;
          };
        }
      );

      legacyPackages = forAllSystems (
        pkgs:
        let
          system = pkgs.stdenv.hostPlatform.system;
          builtForLinux =
            build
              nixpkgs.legacyPackages.${builtins.replaceStrings [ "darwin" ] [ "linux" ] system};
          inherit (self.checks.${system}) integration;
        in
        pkgs.lib.optionalAttrs pkgs.stdenv.hostPlatform.isDarwin {
          vmChecks = {
            e2e = pkgs.linkFarm "e2e" {
              inherit integration;
              inherit (builtForLinux) endToEnd;
            };
            nixos = pkgs.linkFarm "nixos" {
              inherit integration;
              vm = vmTest pkgs builtForLinux;
            };
          };
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
            pkgs.sccache
          ];

          RUSTC_WRAPPER = "sccache";
        };
      });
    };
}
