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
            nativeCheckInputs = [
              pkgs.postgresql_17
              pkgs.mailpit
            ];
            preCheck = ''
              export PGDATA="$TMPDIR/pgdata" PGHOST="$TMPDIR/pgsocket" PGUSER=adunata PGDATABASE=postgres
              mkdir -p "$PGHOST"
              initdb --username="$PGUSER" --auth=trust --no-instructions >/dev/null
              pg_ctl start --wait --silent --options="-k $PGHOST -c listen_addresses="
              export ADUNATA_TEST_MAILPIT_SMTP_PORT=11025 ADUNATA_TEST_MAILPIT_HTTP_PORT=18025
              mailpit --smtp "127.0.0.1:$ADUNATA_TEST_MAILPIT_SMTP_PORT" --listen "127.0.0.1:$ADUNATA_TEST_MAILPIT_HTTP_PORT" --quiet &
              mailpitPid=$!
              for _ in $(seq 50); do (echo > "/dev/tcp/127.0.0.1/$ADUNATA_TEST_MAILPIT_HTTP_PORT") 2>/dev/null && break; sleep 0.1; done
            '';
            postCheck = ''
              kill "$mailpitPid"
              pg_ctl stop --wait --silent
            '';
            preBuild = ''
              mkdir -p ../client
              ln -sfn ${client} ../client/dist
            '';
          };
          cargoArtifacts = craneLib.buildDepsOnly serverArgs;
          server = craneLib.buildPackage (serverArgs // { inherit cargoArtifacts; });
          openapi = pkgs.runCommand "adunata-openapi" { } "${server}/bin/adunata openapi > $out";

          # Starts PostgreSQL, Mailpit and the release binary on port 18080 and leaves the binary's PID in $adunataPid.
          startInstance = ''
            export PGDATA="$TMPDIR/pgdata" PGHOST="$TMPDIR/pgsocket" PGUSER=adunata
            mkdir -p "$PGHOST"
            initdb --username="$PGUSER" --auth=trust --no-instructions >/dev/null
            pg_ctl start --wait --silent --options="-k $PGHOST -c listen_addresses="
            mailpit --smtp 127.0.0.1:11025 --listen 127.0.0.1:18025 --quiet &
            ADUNATA_DATABASE_URL="postgres:///postgres?host=$PGHOST&user=$PGUSER" \
              ADUNATA_LISTEN_ADDRESS=127.0.0.1:18080 ADUNATA_PUBLIC_URL=http://localhost:18080 \
              ADUNATA_SMTP_HOST=127.0.0.1 ADUNATA_SMTP_PORT=11025 ADUNATA_SMTP_ENCRYPTION=none \
              ADUNATA_MAIL_SENDER="Adunata <adunata@example.org>" \
              ${server}/bin/adunata &
            adunataPid=$!
            for _ in $(seq 100); do (echo > /dev/tcp/127.0.0.1/18080) 2>/dev/null && break; sleep 0.1; done
          '';
        in
        {
          inherit client server;

          clippy = craneLib.cargoClippy (
            serverArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          rustFormatting = craneLib.cargoFmt { src = serverSource; };

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

          clientTest = clientPackage {
            pname = "adunata-client-test";
            buildPhase = "npm test";
            installPhase = "touch $out";
          };

          memory =
            pkgs.runCommand "check-memory"
              {
                nativeBuildInputs = [
                  pkgs.postgresql_17
                  pkgs.mailpit
                  pkgs.oha
                ];
                SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
              }
              ''
                ${startInstance}

                oha --no-tui -z 30s -c 5 -m POST \
                  -H 'content-type: application/json' \
                  -d '{"name":"Load","attendance":"accepted","beds":1}' \
                  http://127.0.0.1:18080/api/spike-responses > post.txt &
                postPid=$!
                oha --no-tui -z 30s -c 5 http://127.0.0.1:18080/ > get.txt
                wait "$postPid"

                peak=$(awk '/VmHWM/ { print $2 }' "/proc/$adunataPid/status")
                echo "Peak resident memory: $((peak / 1024)) MB"
                grep -E 'Requests/sec|Success rate' post.txt get.txt
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
            ADUNATA_E2E_URL = "http://127.0.0.1:18080";
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
            contract
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
        // pkgs.lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux { inherit (built) memory endToEnd; }
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
          PGUSER = "adunata";
          PGPASSWORD = "adunata";
          PGDATABASE = "adunata";
          MP_SMTP_BIND_ADDR = "localhost:1025";
          MP_UI_BIND_ADDR = "localhost:8025";

          ADUNATA_PUBLIC_URL = "http://localhost:8080";
          ADUNATA_DATABASE_URL = "postgres://adunata:adunata@localhost/adunata";
          ADUNATA_SMTP_HOST = "localhost";
          ADUNATA_SMTP_PORT = "1025";
          ADUNATA_SMTP_ENCRYPTION = "none";
          ADUNATA_MAIL_SENDER = "Adunata <adunata@localhost>";
        };
      });
    };
}
