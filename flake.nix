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

      version = (fromTOML (builtins.readFile ./server/Cargo.toml)).package.version;

      nixpkgsOnlyLicenses = [
        "GPL-2.0-or-later"
        "GPL-3.0-or-later"
        "LGPL-2.0-or-later"
        "LGPL-3.0-or-later"
        "PostgreSQL"
      ];

      toolsOf = pkgs: {
        inherit (pkgs)
          binutils
          cargo
          cargo-about
          cargo-deny
          clippy
          editorconfig-checker
          mailpit
          nixfmt
          nodejs_26
          openssl
          patchelf
          pnpm
          postgresql_17
          rust-analyzer
          rustc
          rustfmt
          sccache
          skopeo
          zizmor
          ;
        rustc-prebuilt = pkgs.rust.packages.prebuilt.rustc;
      };

      build =
        pkgs:
        let
          craneLib = crane.mkLib pkgs;
          tools = toolsOf pkgs;
          clientSourceWith =
            extraFiles:
            pkgs.lib.fileset.toSource {
              root = ./.;
              fileset = pkgs.lib.fileset.intersection (pkgs.lib.fileset.gitTracked ./.) (
                pkgs.lib.fileset.unions (
                  [
                    ./client
                    ./server/locales
                  ]
                  ++ extraFiles
                )
              );
            };
          clientSource = clientSourceWith [ ];

          pnpmFetcherVersion = 5;

          pnpmDepsInputsDigest = builtins.substring 0 8 (
            builtins.hashString "sha256" (
              builtins.readFile ./client/pnpm-lock.yaml
              + builtins.readFile ./client/pnpm-workspace.yaml
              + tools.pnpm.version
              + toString pnpmFetcherVersion
            )
          );

          pnpmDeps = pkgs.fetchPnpmDeps {
            pname = "adunata-client-${pnpmDepsInputsDigest}";
            src = clientSource;
            sourceRoot = "source/client";
            inherit (tools) pnpm;
            fetcherVersion = pnpmFetcherVersion;
            hash = pkgs.lib.trim (builtins.readFile ./client/pnpm-deps.hash);
          };

          clientPackage =
            args:
            pkgs.stdenv.mkDerivation (
              {
                pname = "adunata-client";
                inherit version;
                src = clientSource;
                sourceRoot = "source/client";
                nativeBuildInputs = [
                  tools.nodejs_26
                  tools.pnpm
                  pkgs.pnpmConfigHook
                ];
                inherit pnpmDeps;
                buildPhase = "pnpm run build";
              }
              // args
            );

          client = clientPackage {
            installPhase = "cp -r dist $out";
          };

          cargoSources = pkgs.lib.fileset.difference (craneLib.fileset.commonCargoSources ./server) (
            pkgs.lib.fileset.unions [
              ./server/deny.toml
              ./server/about.toml
            ]
          );

          crateNotices = craneLib.mkCargoDerivation {
            src = pkgs.lib.fileset.toSource {
              root = ./server;
              fileset = pkgs.lib.fileset.unions [
                ./server/Cargo.toml
                ./server/Cargo.lock
                ./server/about.toml
                ./server/about.hbs
              ];
            };
            postPatch = "mkdir src && touch src/main.rs";
            pnameSuffix = "-crate-notices";
            cargoArtifacts = null;
            doInstallCargoArtifacts = false;
            nativeBuildInputs = [ tools.cargo-about ];
            buildPhaseCargoCommand = "cargo about generate --frozen --fail about.hbs --output-file licenses.txt";
            installPhaseCommand = "install -D licenses.txt $out/licenses.txt";
          };

          clientWithNotices = pkgs.runCommand "adunata-client-with-notices" { } ''
            cp -r ${client} $out
            chmod -R u+w $out
            cat ${crateNotices}/licenses.txt >> $out/licenses.txt
            printf '\nThe binary was built with Rust %s. licenses-rust.html next to this file holds the copyright notices of its standard library.\n' ${tools.rustc.version} >> $out/licenses.txt
            cp ${rustStandardLibraryNotices} $out/licenses-rust.html
          '';

          rustStandardLibraryNotices = "${tools.rustc-prebuilt.unwrapped}/share/doc/rust/COPYRIGHT-library.html";

          serverSource = pkgs.lib.fileset.toSource {
            root = ./server;
            fileset = pkgs.lib.fileset.unions [
              cargoSources
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
              ln -sfn ${clientWithNotices} ../client/dist
            '';
          };
          testServices = {
            # The tests bind PostgreSQL and Mailpit to the loopback address, which the macOS sandbox forbids otherwise.
            __darwinAllowLocalNetworking = true;
            nativeCheckInputs = [
              tools.postgresql_17
              tools.mailpit
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

          system = pkgs.stdenv.hostPlatform.system;

          release = pkgs.runCommand "adunata-release-${system}" { nativeBuildInputs = [ tools.patchelf ]; } ''
            mkdir $out
            cp ${server}/bin/adunata $out/adunata
            chmod u+w $out/adunata
            patchelf --set-interpreter /${pkgs.stdenv.hostPlatform.libDir}/${builtins.baseNameOf pkgs.stdenv.cc.bintools.dynamicLinker} --remove-rpath $out/adunata
            cp ${./deploy/systemd/adunata.service} $out/adunata.service
            cp ${./deploy/systemd/adunata.env.example} $out/adunata.env.example
            cp ${./LICENSE} $out/LICENSE
            cp ${clientWithNotices}/licenses.txt ${clientWithNotices}/licenses-rust.html $out/
          '';

          releaseArchive = pkgs.runCommand "adunata-release-archive-${system}" { } ''
            mkdir $out
            archive=$out/adunata-${system}.tar.gz
            (cd ${release} && export LC_ALL=C && tar --create --sort=name --owner=0 --group=0 --numeric-owner --mtime=@0 --format=gnu -- *) | gzip --no-name --best > $archive
            cd $out
            sha256sum adunata-${system}.tar.gz > adunata-${system}.tar.gz.sha256
          '';

          releaseLinkage =
            pkgs.runCommand "check-release-linkage" { nativeBuildInputs = [ tools.binutils ]; }
              ''
                binary=${release}/adunata
                readelf --dynamic $binary > dynamic
                if grep --extended-regexp 'RPATH|RUNPATH' dynamic; then
                  echo "The release binary must find its libraries on the distribution, not in the Nix store."
                  exit 1
                fi
                libraries=$(grep --only-matching 'Shared library: \[[^]]*\]' dynamic | grep --invert-match ld-linux | sed 's/.*\[\(.*\)\]/\1/' | sort | xargs)
                echo "Libraries: $libraries"
                if [ "$libraries" != "libc.so.6 libgcc_s.so.1 libm.so.6" ]; then
                  echo "The release binary may only need glibc and libgcc_s, which every supported distribution ships."
                  exit 1
                fi
                for need in $(readelf --version-info $binary | sed --quiet '/Version needs/,$p' | grep --only-matching 'Name: [^ ]*' | cut --delimiter=' ' --fields=2 | sort --unique); do
                  case $need in
                    GLIBC_[0-9]*) newest=GLIBC_2.34 ;;
                    GCC_[0-9]*) newest=GCC_4.2.0 ;;
                    *) echo "The release binary needs $need, which the supported distributions may lack."; exit 1 ;;
                  esac
                  if [ "$(printf '%s\n' "$need" "$newest" | sort --version-sort | tail -1)" != "$newest" ]; then
                    echo "The release binary needs $need, newer than the supported $newest."
                    exit 1
                  fi
                done
                touch $out
              '';

          imagePackages = [
            {
              name = "glibc";
              package = pkgs.glibc;
              licenseFiles = [
                "LGPL-2.1.txt"
                "GPL-2.0.txt"
                "glibc-LICENSES.txt"
              ];
            }
            {
              name = "gcc";
              package = pkgs.stdenv.cc.cc;
              licenseFiles = [
                "GPL-3.0.txt"
                "GCC-exception-3.1.txt"
              ];
            }
            {
              name = "xgcc";
              package = pkgs.stdenv.cc.cc;
              licenseFiles = [
                "GPL-3.0.txt"
                "GCC-exception-3.1.txt"
              ];
            }
            {
              name = "libidn2";
              package = pkgs.libidn2;
              licenseFiles = [
                "LGPL-3.0.txt"
                "GPL-3.0.txt"
                "GPL-2.0.txt"
              ];
              source = {
                version = "2.3.8";
                url = "https://ftp.gnu.org/gnu/libidn/libidn2-2.3.8.tar.gz";
                outputHash = "sha256-9VeRG/YXFiHh9y/zX1sYJbs1tS7UUyXc3ukx5dPAeHo=";
              };
            }
            {
              name = "libunistring";
              package = pkgs.libunistring;
              licenseFiles = [
                "LGPL-3.0.txt"
                "GPL-3.0.txt"
              ];
            }
            {
              name = "nss-cacert";
              package = pkgs.cacert;
              licenseFiles = [ "MPL-2.0.txt" ];
            }
          ];

          versionOf = entry: (builtins.parseDrvName entry.package.name).version;
          licenseIdsOf = entry: map (license: license.spdxId) (pkgs.lib.toList entry.package.meta.license);

          imagePackageNotice =
            entry:
            let
              source = entry.source or entry.package.src;
              url = source.url or (builtins.head source.urls);
            in
            assert pkgs.lib.assertMsg ((source.version or (versionOf entry)) == versionOf entry)
              "The source of ${entry.name} in imagePackages names version ${source.version or ""}, but the image contains ${versionOf entry}. Update url, outputHash and version of the source.";
            ''
              mkdir -p "$dir/${entry.name}"
              ${pkgs.lib.concatMapStrings (file: ''
                cp ${./nix/licenses}/${file} "$dir/${entry.name}/"
              '') entry.licenseFiles}
              cat > "$dir/${entry.name}/SOURCE" <<'EOF'
              Package: ${entry.name} ${versionOf entry}
              Licenses: ${pkgs.lib.concatStringsSep ", " (licenseIdsOf entry)}
              Built by nixpkgs revision ${nixpkgs.rev} from the source
              ${url}
              with the hash ${source.outputHash}.
              The revision of nixpkgs names every patch applied to that source.
              EOF
            '';

          imageNotices = pkgs.runCommand "adunata-image-notices" { } ''
            dir=$out/share/licenses
            mkdir -p "$dir"
            cp ${clientWithNotices}/licenses.txt ${clientWithNotices}/licenses-rust.html "$dir/"
            ${pkgs.lib.concatMapStrings imagePackageNotice imagePackages}
          '';

          imageRoots = [
            server
            pkgs.cacert
          ];

          imageClosure =
            pkgs.runCommand "check-image-closure"
              {
                closure = pkgs.closureInfo { rootPaths = imageRoots; };
                expected = pkgs.lib.concatMapStrings (entry: "${entry.name}-${versionOf entry}\n") imagePackages;
              }
              ''
                sed --regexp-extended 's|^/nix/store/[a-z0-9]+-||; s/^(.*-[0-9][^-]*)-[a-z]+$/\1/' $closure/store-paths \
                  | grep --invert-match '^adunata-' | sort --unique > actual
                printf '%s' "$expected" | sort --unique > expected
                if ! diff expected actual; then
                  echo "The packages from nixpkgs in the image differ from imagePackages in flake.nix. Lines with > are in the image without an entry, lines with < are entries that the image no longer contains."
                  exit 1
                fi
                touch $out
              '';

          image = pkgs.dockerTools.buildLayeredImage {
            name = "adunata";
            tag = "latest";
            extraCommands = ''
              mkdir share
              cp --recursive ${imageNotices}/share/licenses share/licenses
              chmod --recursive u+w share/licenses
            '';
            config = {
              Cmd = [
                "adunata"
                "serve"
              ];
              Env = [
                "ADNT_HTTP_LISTEN_ADDRESS=0.0.0.0:8080"
                "PATH=${server}/bin"
                "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
              ];
              ExposedPorts."8080/tcp" = { };
              User = "65534:65534";
            };
          };
        in
        {
          inherit
            client
            server
            release
            releaseArchive
            releaseLinkage
            image
            imageClosure
            imagePackages
            ;

          clippy = craneLib.cargoClippy (
            serverArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          rustFormatting = craneLib.cargoFmt { src = serverSource; };

          rustDependencies = craneLib.cargoDeny {
            src = pkgs.lib.fileset.toSource {
              root = ./server;
              fileset = craneLib.fileset.commonCargoSources ./server;
            };
          };

          serverUnitTests =
            serverTests serverArgs "-unit-tests"
              "cargoWithProfile test --locked --bins -- --skip integration::";

          serverIntegrationTests =
            serverTests
              (
                serverArgs
                // testServices
                // {
                  ADUNATA_OPENAPI_DOCUMENT = ./client/openapi.json;
                }
              )
              "-integration-tests"
              ''
                cargoWithProfile test --locked --bins -- integration::
                ADUNATA_TEST_BINARY=${server}/bin/adunata cargoWithProfile test --locked --test startup
              '';

          contract = clientPackage {
            pname = "adunata-contract";
            buildPhase = "sh scripts/check-generated.sh";
            installPhase = "touch $out";
          };

          clientUnitTests = clientPackage {
            pname = "adunata-client-unit-tests";
            buildPhase = "pnpm test";
            installPhase = "touch $out";
          };

          clientIntegrationTests = clientPackage {
            pname = "adunata-client-integration-tests";
            buildPhase = "pnpm run test:integration";
            installPhase = "touch $out";
          };

          clientLicenses = clientPackage {
            pname = "adunata-client-licenses";
            src = clientSourceWith [ ./server/deny.toml ];
            buildPhase = "pnpm run lint:licenses";
            installPhase = "touch $out";
          };

          clientLint = clientPackage {
            pname = "adunata-client-lint";
            buildPhase = "pnpm run lint";
            installPhase = "touch $out";
          };
        };

      vmTest =
        pkgs: built:
        import ./nix/test.nix {
          inherit pkgs toolsOf;
          inherit (built) release image;
          module = self.nixosModules.default;
        };
    in
    {
      formatter = forAllSystems (pkgs: (toolsOf pkgs).nixfmt);

      packages = forAllSystems (
        pkgs:
        let
          built = build pkgs;
        in
        {
          default = built.server;
        }
        // pkgs.lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
          inherit (built) release releaseArchive image;
        }
      );

      nixosModules.default = import ./nix/module.nix (pkgs: (build pkgs).server);

      checks = forAllSystems (
        pkgs:
        let
          built = build pkgs;
          builtForLinux =
            build
              nixpkgs.legacyPackages.${
                builtins.replaceStrings [ "darwin" ] [ "linux" ] pkgs.stdenv.hostPlatform.system
              };
          tools = toolsOf pkgs;
        in
        let
          lintChecks = {
            inherit (built)
              clippy
              rustFormatting
              rustDependencies
              clientLint
              ;

            rustToolchain = pkgs.runCommand "check-rust-toolchain" { nativeBuildInputs = [ tools.rustc ]; } ''
              pinned=$(sed --quiet 's/^channel = "\(.*\)"$/\1/p' ${./server/rust-toolchain.toml})
              if [ "$(rustc --version | cut --delimiter=' ' --fields=2)" != "$pinned" ]; then
                echo "server/rust-toolchain.toml pins Rust $pinned, but nixpkgs ships $(rustc --version). Set the channel to the version of nixpkgs."
                exit 1
              fi
              touch $out
            '';

            editorconfigCheckerVersion = pkgs.runCommand "check-editorconfig-checker-version" { } ''
              pinned=$(sed --quiet 's/.*EC_VERSION=v\([0-9.]*\) .*/\1/p' ${./client/package.json})
              if [ "$pinned" != "${tools.editorconfig-checker.version}" ]; then
                echo "pnpm run lint:editorconfig pins editorconfig-checker $pinned, but nixpkgs ships ${tools.editorconfig-checker.version}. Set EC_VERSION in client/package.json to the version of nixpkgs."
                exit 1
              fi
              touch $out
            '';

            pnpmVersion =
              let
                pinned =
                  (builtins.fromJSON (builtins.readFile ./client/package.json)).packageManager or "no version";
              in
              pkgs.runCommand "check-pnpm-version" { } (
                if pinned == "pnpm@${tools.pnpm.version}" then
                  "touch $out"
                else
                  ''
                    echo "packageManager in client/package.json pins ${pinned}, but nixpkgs ships pnpm ${tools.pnpm.version}. Set packageManager to pnpm@${tools.pnpm.version}."
                    exit 1
                  ''
              );

            inherit (built) clientLicenses;

            projectVersion =
              let
                lockedVersion =
                  (pkgs.lib.findFirst (crate: crate.name == "adunata") {
                    version = "no version";
                  } (fromTOML (builtins.readFile ./server/Cargo.lock)).package).version;
                clientVersion =
                  (builtins.fromJSON (builtins.readFile ./client/package.json)).version or "no version";
              in
              pkgs.runCommand "check-project-version" { } (
                if lockedVersion != version || clientVersion != version then
                  ''
                    echo "server/Cargo.toml names version ${version}, server/Cargo.lock ${lockedVersion} and client/package.json ${clientVersion}. Set all three to the same version."
                    exit 1
                  ''
                else if pkgs.lib.hasInfix "+" version then
                  ''
                    echo "The version ${version} has build metadata, which an image tag cannot hold. Remove the part from +."
                    exit 1
                  ''
                else
                  "touch $out"
              );

            cargoDenyVersion = pkgs.runCommand "check-cargo-deny-version" { } ''
              pinned=$(sed --quiet 's/.*cargo-deny@\([0-9.]*\).*/\1/p' ${./README.md})
              if [ "$pinned" != "${tools.cargo-deny.version}" ]; then
                echo "The README pins cargo-deny $pinned, but nixpkgs ships ${tools.cargo-deny.version}. Set the version in the README to the version of nixpkgs."
                exit 1
              fi
              touch $out
            '';

            workflows =
              let
                source = pkgs.lib.fileset.toSource {
                  root = ./.;
                  fileset = ./.github;
                };
              in
              pkgs.runCommand "check-workflows" { nativeBuildInputs = [ tools.zizmor ]; } ''
                zizmor --offline --no-progress ${source}/.github
                touch $out
              '';

            cargoAboutVersion = pkgs.runCommand "check-cargo-about-version" { } ''
              pinned=$(sed --quiet 's/.*cargo-about@\([0-9.]*\).*/\1/p' ${./README.md})
              if [ "$pinned" != "${tools.cargo-about.version}" ]; then
                echo "The README pins cargo-about $pinned, but nixpkgs ships ${tools.cargo-about.version}. Set the version in the README to the version of nixpkgs."
                exit 1
              fi
              touch $out
            '';

            rustStandardLibraryNoticesVersion =
              pkgs.runCommand "check-rust-standard-library-notices-version" { }
                (
                  if tools.rustc-prebuilt.version == tools.rustc.version then
                    "touch $out"
                  else
                    ''
                      echo "The notices of the Rust standard library come from the prebuilt rustc ${tools.rustc-prebuilt.version}, but the server is built with Rust ${tools.rustc.version}. Take COPYRIGHT-library.html of the release of Rust ${tools.rustc.version} instead."
                      exit 1
                    ''
                );

            crateNoticeLicenses =
              let
                allowed =
                  file: key:
                  pkgs.lib.sort pkgs.lib.lessThan (pkgs.lib.attrByPath key [ ] (fromTOML (builtins.readFile file)));
              in
              pkgs.runCommand "check-crate-notice-licenses" { } (
                if
                  allowed ./server/about.toml [ "accepted" ] == allowed ./server/deny.toml [
                    "licenses"
                    "allow"
                  ]
                then
                  "touch $out"
                else
                  ''
                    echo "accepted in server/about.toml differs from licenses.allow in server/deny.toml. Keep both lists equal."
                    exit 1
                  ''
              );

            toolLicenses =
              let
                allowed = (fromTOML (builtins.readFile ./server/deny.toml)).licenses.allow ++ nixpkgsOnlyLicenses;
                packages = builtins.attrValues tools ++ map (entry: entry.package) builtForLinux.imagePackages;
                problemsOf =
                  package:
                  let
                    licenses = pkgs.lib.toList (package.meta.license or [ ]);
                  in
                  if licenses == [ ] then
                    [ "${package.name} declares no license" ]
                  else
                    pkgs.lib.concatMap (
                      license:
                      if !(license ? spdxId) then
                        [ "${package.name} declares ${license.shortName or "a license"} without an SPDX identifier" ]
                      else if builtins.elem license.spdxId allowed then
                        [ ]
                      else
                        [ "${package.name} is under ${license.spdxId}, which no allowed list names" ]
                    ) licenses;
                problems = pkgs.lib.concatMap problemsOf packages;
              in
              pkgs.runCommand "check-tool-licenses" { } (
                if problems == [ ] then
                  "touch $out"
                else
                  ''
                    printf '%s\n' ${pkgs.lib.escapeShellArgs problems}
                    echo "Every tool from nixpkgs needs a license from server/deny.toml or from nixpkgsOnlyLicenses in flake.nix."
                    exit 1
                  ''
              );

            nixFormatting = pkgs.runCommand "check-nix-formatting" { nativeBuildInputs = [ tools.nixfmt ]; } ''
              find ${self} -name '*.nix' -exec nixfmt --check {} +
              touch $out
            '';

            editorconfig =
              pkgs.runCommand "check-editorconfig" { nativeBuildInputs = [ tools.editorconfig-checker ]; }
                ''
                  cd ${self}
                  editorconfig-checker
                  touch $out
                '';
          };
          buildChecks = {
            inherit (built) server;
          }
          // pkgs.lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
            inherit (built) releaseLinkage imageClosure;
          };
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
        // pkgs.lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux rec {
          nixosVmTest = vmTest pkgs built;
          nixos = pkgs.linkFarm "nixos" {
            inherit integration;
            vm = nixosVmTest;
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
            nixos = pkgs.linkFarm "nixos" {
              inherit integration;
              vm = vmTest pkgs builtForLinux;
            };
          };
        }
      );

      devShells = forAllSystems (
        pkgs:
        let
          tools = toolsOf pkgs;
        in
        {
          default = pkgs.mkShell {
            packages = [
              tools.cargo
              tools.cargo-about
              tools.cargo-deny
              tools.rustc
              tools.clippy
              tools.rustfmt
              tools.rust-analyzer
              tools.nodejs_26
              tools.pnpm
              tools.sccache
            ];

            RUSTC_WRAPPER = "sccache";
          };
        }
      );
    };
}
