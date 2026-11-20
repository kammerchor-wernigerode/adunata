{
  pkgs,
  toolsOf,
  module,
  release,
  image,
}:
let
  testCertificates =
    pkgs.runCommand "adunata-test-certificates" { nativeBuildInputs = [ (toolsOf pkgs).openssl ]; }
      ''
        mkdir $out
        openssl req -x509 -newkey ec -pkeyopt ec_paramgen_curve:prime256v1 -nodes -days 3650 \
          -subj "/CN=Adunata test authority" -keyout authority.key -out $out/authority.crt
        printf 'subjectAltName=IP:127.0.0.1\nbasicConstraints=CA:FALSE\nextendedKeyUsage=serverAuth\n' > server.ext
        for server in mailpit postgresql; do
          openssl req -newkey ec -pkeyopt ec_paramgen_curve:prime256v1 -nodes \
            -subj "/CN=127.0.0.1" -keyout $out/$server.key -out $server.csr
          openssl x509 -req -in $server.csr -CA $out/authority.crt -CAkey authority.key -CAcreateserial -days 3650 \
            -extfile server.ext -out $out/$server.crt
        done
      '';

  mailpitWithImplicitTls =
    { pkgs, ... }:
    {
      systemd.services.mailpit = {
        wantedBy = [ "multi-user.target" ];
        serviceConfig.ExecStart = "${(toolsOf pkgs).mailpit}/bin/mailpit --smtp 127.0.0.1:1465 --listen 127.0.0.1:8025 --smtp-tls-cert ${testCertificates}/mailpit.crt --smtp-tls-key ${testCertificates}/mailpit.key --smtp-require-tls";
      };
    };

  postgresqlWithTls = {
    services.postgresql = {
      enableTCPIP = true;
      authentication = "host adunata adunata 127.0.0.0/8 trust";
      settings = {
        ssl = true;
        ssl_cert_file = "${testCertificates}/postgresql.crt";
        ssl_key_file = "/run/postgresql-tls/postgresql.key";
      };
    };
    systemd.tmpfiles.rules = [
      "d /run/postgresql-tls 0755 root root -"
      "C /run/postgresql-tls/postgresql.key - - - - ${testCertificates}/postgresql.key"
      "z /run/postgresql-tls/postgresql.key 0640 root postgres -"
    ];
  };

  trustsTestAuthority = {
    security.pki.certificateFiles = [ "${testCertificates}/authority.crt" ];
  };
in
pkgs.testers.runNixOSTest {
  name = "adunata";

  nodes = {
    nixos =
      { pkgs, ... }:
      {
        imports = [
          module
          mailpitWithImplicitTls
          postgresqlWithTls
          trustsTestAuthority
        ];
        services.adunata = {
          enable = true;
          http.publicUrl = "http://localhost:8080";
          localization.timeZone = "Europe/Berlin";
          mail.sender = "Adunata <adunata@example.org>";
          smtp = {
            host = "127.0.0.1";
            port = 1465;
            encryption = "tls";
          };
          bootstrap = {
            organizerMail = "organizer@example.org";
            organizerFirstName = "Ada";
            organizerLastName = "Lovelace";
          };
        };
      };

    binary =
      { pkgs, ... }:
      {
        imports = [
          mailpitWithImplicitTls
          trustsTestAuthority
        ];
        environment.ldso = pkgs.stdenv.cc.bintools.dynamicLinker;
        systemd.services.adunata = {
          overrideStrategy = "asDropin";
          environment.LD_LIBRARY_PATH = "${pkgs.stdenv.cc.cc.lib}/lib";
        };
        services.postgresql = {
          enable = true;
          enableTCPIP = true;
          ensureDatabases = [ "adunata" ];
          ensureUsers = [
            {
              name = "adunata";
              ensureDBOwnership = true;
            }
          ];
          authentication = "host adunata adunata 127.0.0.1/32 trust";
        };
      };

    container =
      { pkgs, ... }:
      {
        imports = [
          mailpitWithImplicitTls
          postgresqlWithTls
        ];
        virtualisation.podman.enable = true;
        virtualisation.memorySize = 2048;
        services.postgresql = {
          enable = true;
          ensureDatabases = [ "adunata" ];
          ensureUsers = [
            {
              name = "adunata";
              ensureDBOwnership = true;
            }
          ];
        };
      };
  };

  testScript = ''
    import json

    def assert_serves_adunata(machine):
        health = json.loads(machine.succeed("curl --fail --silent http://127.0.0.1:8080/api/health"))
        assert health == {"status": "ready"}, health
        page = machine.succeed("curl --fail --silent http://127.0.0.1:8080/")
        assert "<title>Adunata</title>" in page, page

    def assert_serves_notices(machine):
        notices = machine.succeed("curl --fail --silent http://127.0.0.1:8080/licenses.txt")
        assert "\n- axum " in notices, "the notices lack the crates"
        assert "\nreact " in notices, "the notices lack the client"
        rust = machine.succeed("curl --fail --silent http://127.0.0.1:8080/licenses-rust.html")
        assert "hashbrown" in rust, "the notices lack the standard library"

    def request_sign_in(machine):
        machine.wait_for_open_port(1465)
        machine.succeed(
            "curl --fail --silent -H 'content-type: application/json'"
            " -d '{\"mail\":\"organizer@example.org\"}' http://127.0.0.1:8080/api/sign-in-requests"
        )

    def database_connection_encrypted(machine):
        encrypted = machine.succeed(
            "sudo -u postgres psql -d adunata -tAc"
            " \"SELECT ssl FROM pg_stat_ssl JOIN pg_stat_activity USING (pid) WHERE usename = 'adunata' ORDER BY backend_start DESC LIMIT 1\""
        ).strip()
        assert encrypted in ("t", "f"), encrypted
        return encrypted == "t"

    def connect_nixos_through(url):
        nixos.succeed(
            "mkdir -p /run/systemd/system/adunata.service.d",
            f"printf '[Service]\\nEnvironment=ADNT_DATABASE_URL={url}\\n' > /run/systemd/system/adunata.service.d/database.conf",
            "systemctl daemon-reload",
            "systemctl restart adunata.service",
        )
        nixos.wait_for_open_port(8080)
        assert_serves_adunata(nixos)
        assert database_connection_encrypted(nixos), url

    def assert_mail_arrives(machine):
        machine.wait_until_succeeds(
            "curl --fail --silent http://127.0.0.1:8025/api/v1/messages | grep --quiet '\"total\":1'", timeout=90
        )

    start_all()

    with subtest("Install on NixOS"):
        nixos.wait_for_unit("adunata.service")
        nixos.wait_for_open_port(8080)
        assert_serves_adunata(nixos)
        assert_serves_notices(nixos)
        applied = nixos.succeed("sudo -u postgres psql -d adunata -tAc 'SELECT count(*) FROM _sqlx_migrations WHERE success'")
        assert int(applied) >= 2, applied
        exposure = nixos.succeed("systemd-analyze security adunata.service | tail -1")
        print(exposure)
        assert not database_connection_encrypted(nixos), "the Unix socket must stay plain"

    with subtest("Mail over TLS on NixOS, trusted through the host's store"):
        request_sign_in(nixos)
        assert_mail_arrives(nixos)

    with subtest("Database over TLS on NixOS, verified with the authority in sslrootcert"):
        connect_nixos_through("postgres://adunata@127.0.0.1/adunata?sslmode=verify-full&sslrootcert=${testCertificates}/authority.crt")

    with subtest("Database over TLS on NixOS, verified through the host's store"):
        connect_nixos_through("postgres://adunata@127.0.0.1/adunata?sslmode=verify-full")

    with subtest("Database over TLS on NixOS without TLS parameters"):
        connect_nixos_through("postgres://adunata@127.0.0.1/adunata")

    with subtest("Install from release binary"):
        binary.wait_for_unit("postgresql.target")
        binary.succeed(
            "install -D -m 0755 ${release}/adunata /usr/local/bin/adunata",
            "mkdir -p /etc/adunata /run/systemd/system",
            "cp ${release}/adunata.service /run/systemd/system/adunata.service",
            "sed -e 's#adunata@localhost#adunata@127.0.0.1#' -e 's#mail.example.org#127.0.0.1#' -e 's#https://adunata.example.org#http://localhost:8080#' -e '/ADNT_SMTP_USERNAME/d' ${release}/adunata.env.example > /etc/adunata/adunata.env",
            "printf '%s\\n' ADNT_SMTP_PORT=1465 ADNT_SMTP_ENCRYPTION=tls ADNT_BOOTSTRAP_ORGANIZER_MAIL=organizer@example.org ADNT_BOOTSTRAP_ORGANIZER_FIRST_NAME=Ada ADNT_BOOTSTRAP_ORGANIZER_LAST_NAME=Lovelace >> /etc/adunata/adunata.env",
            "touch /etc/adunata/database-password /etc/adunata/smtp-password",
            "systemctl daemon-reload",
            "systemctl start adunata.service",
        )
        binary.wait_for_open_port(8080)
        assert_serves_adunata(binary)
        assert_serves_notices(binary)
        binary.succeed(
            "curl --fail --silent http://127.0.0.1:8080/licenses.txt | cmp - ${release}/licenses.txt",
            "curl --fail --silent http://127.0.0.1:8080/licenses-rust.html | cmp - ${release}/licenses-rust.html",
        )

    with subtest("Mail over TLS from the release binary, trusted through the host's store"):
        request_sign_in(binary)
        assert_mail_arrives(binary)

    with subtest("Release binary refuses a database without TLS under sslmode=require"):
        binary.succeed(
            "sed -i 's#^ADNT_DATABASE_URL=.*#&?sslmode=require#' /etc/adunata/adunata.env",
            "systemctl restart adunata.service",
        )
        binary.wait_until_succeeds("systemctl show adunata.service --property=ExecMainStatus | grep -x ExecMainStatus=1")
        binary.succeed("journalctl -u adunata.service | grep 'database is not ready' | grep --quiet -i tls")
        binary.succeed("systemctl stop adunata.service")

    with subtest("Install as container"):
        container.wait_for_unit("postgresql.target")
        container.succeed("podman load --input ${image}")
        container_settings = (
            " --network host"
            " --env ADNT_HTTP_LISTEN_ADDRESS=127.0.0.1:8080"
            " --env ADNT_HTTP_PUBLIC_URL=http://localhost:8080"
            " --env ADNT_SMTP_HOST=127.0.0.1"
            " --env ADNT_SMTP_PORT=1465"
            " --env ADNT_SMTP_ENCRYPTION=tls"
            " --env 'ADNT_MAIL_SENDER=Adunata <adunata@example.org>'"
            " --env ADNT_LOCALIZATION_TIME_ZONE=Europe/Berlin"
            " --env ADNT_BOOTSTRAP_ORGANIZER_MAIL=organizer@example.org"
            " --env ADNT_BOOTSTRAP_ORGANIZER_FIRST_NAME=Ada"
            " --env ADNT_BOOTSTRAP_ORGANIZER_LAST_NAME=Lovelace"
        )
        plain_database = " --env ADNT_DATABASE_URL=postgres://adunata@127.0.0.1/adunata"
        container.succeed("podman run --detach --name untrusting" + container_settings + plain_database + " adunata:latest")
        container.wait_for_open_port(8080)
        assert_serves_adunata(container)
        assert_serves_notices(container)

    with subtest("Image ships a bundle of certificate authorities"):
        bundle = container.succeed(
            "podman image inspect adunata:latest --format '{{range .Config.Env}}{{println .}}{{end}}'"
            " | sed --quiet 's/^SSL_CERT_FILE=//p'"
        ).strip()
        assert bundle, "the image sets no SSL_CERT_FILE"
        root = container.succeed("podman image mount adunata:latest").strip()
        container.succeed(f"test -s {root}{bundle}")

    with subtest("Image ships the notices of its system packages"):
        licenses = f"{root}/share/licenses"
        container.succeed(f"test -s {licenses}/licenses.txt", f"test -s {licenses}/glibc/SOURCE", f"test -s {licenses}/glibc/LGPL-2.1.txt")

    with subtest("Image rejects a mail server outside its bundle"):
        request_sign_in(container)
        container.wait_until_succeeds("podman logs untrusting 2>&1 | grep 'mail delivery failed' | grep --quiet 'encryption'", timeout=60)
        container.succeed("curl --fail --silent http://127.0.0.1:8025/api/v1/messages | grep --quiet '\"total\":0'")
        container.succeed("podman stop untrusting")
        container.succeed("sudo -u postgres psql -d adunata -c 'UPDATE mail_outbox SET next_attempt_at = now()'")

    with subtest("Image trusts the bundle that SSL_CERT_FILE names"):
        container.succeed(
            "podman run --detach --name trusting"
            " --volume ${testCertificates}/authority.crt:/authority.crt:ro"
            " --env SSL_CERT_FILE=/authority.crt"
            + container_settings + plain_database + " adunata:latest"
        )
        assert_mail_arrives(container)
        container.succeed("podman stop trusting")

    def run_container_with_database(url):
        return container.execute(
            "timeout 60 podman run --rm --volume ${testCertificates}/authority.crt:/authority.crt:ro"
            + container_settings + f" --env 'ADNT_DATABASE_URL={url}' adunata:latest 2>&1"
        )

    with subtest("Image refuses a database outside its bundle"):
        status, output = run_container_with_database("postgres://adunata@127.0.0.1/adunata?sslmode=verify-full")
        assert status == 1, output
        assert "UnknownIssuer" in output, output

    with subtest("Image trusts the database through the authority in sslrootcert"):
        container.succeed(
            "podman run --detach --name verifying --volume ${testCertificates}/authority.crt:/authority.crt:ro"
            + container_settings
            + " --env 'ADNT_DATABASE_URL=postgres://adunata@127.0.0.1/adunata?sslmode=verify-full&sslrootcert=/authority.crt'"
            + " adunata:latest"
        )
        container.wait_for_open_port(8080)
        assert_serves_adunata(container)
        assert database_connection_encrypted(container)
        container.succeed("podman stop verifying")

    with subtest("Image refuses a database whose certificate does not name the host"):
        status, output = run_container_with_database("postgres://adunata@localhost/adunata?sslmode=verify-full&sslrootcert=/authority.crt")
        assert status == 1, output
        assert "not valid for name" in output, output

    with subtest("Image checks only the authority under sslmode=verify-ca"):
        container.succeed(
            "podman run --detach --name verifying-authority --volume ${testCertificates}/authority.crt:/authority.crt:ro"
            + container_settings
            + " --env 'ADNT_DATABASE_URL=postgres://adunata@127.0.0.2/adunata?sslmode=verify-ca&sslrootcert=/authority.crt'"
            + " adunata:latest"
        )
        container.wait_for_open_port(8080)
        assert_serves_adunata(container)
        assert database_connection_encrypted(container)
        container.succeed("podman stop verifying-authority")

    with subtest("Stop gracefully"):
        nixos.succeed("systemctl stop adunata.service")
        nixos.succeed("systemctl show adunata.service --property=ExecMainStatus | grep -x ExecMainStatus=0")

    with subtest("Invalid configuration leaves the service stopped"):
        for machine in [nixos, binary]:
            machine.succeed(
                "mkdir -p /run/systemd/system/adunata.service.d",
                "printf '[Service]\\nEnvironment=ADNT_CONFIG_FILE=/nonexistent/adunata.toml\\n' > /run/systemd/system/adunata.service.d/invalid.conf",
                "systemctl daemon-reload",
                "systemctl restart adunata.service || true",
            )
            machine.wait_until_succeeds("systemctl is-failed adunata.service", timeout=30)
            machine.succeed(
                "systemctl show adunata.service --property=ExecMainStatus | grep -x ExecMainStatus=78",
                "sleep 7",
                "systemctl is-failed adunata.service",
                "systemctl show adunata.service --property=NRestarts | grep -x NRestarts=0",
            )
  '';
}
