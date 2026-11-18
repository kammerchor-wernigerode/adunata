{
  pkgs,
  module,
  release,
  image,
}:
let
  # A certificate authority of its own, which only the hosts that trust it accept, and a certificate for Mailpit at 127.0.0.1.
  certificates =
    pkgs.runCommand "adunata-test-certificates" { nativeBuildInputs = [ pkgs.openssl ]; }
      ''
        mkdir $out
        openssl req -x509 -newkey ec -pkeyopt ec_paramgen_curve:prime256v1 -nodes -days 3650 \
          -subj "/CN=Adunata test authority" -keyout authority.key -out $out/authority.crt
        openssl req -newkey ec -pkeyopt ec_paramgen_curve:prime256v1 -nodes \
          -subj "/CN=127.0.0.1" -keyout $out/mailpit.key -out mailpit.csr
        printf 'subjectAltName=IP:127.0.0.1\nbasicConstraints=CA:FALSE\nextendedKeyUsage=serverAuth\n' > mailpit.ext
        openssl x509 -req -in mailpit.csr -CA $out/authority.crt -CAkey authority.key -CAcreateserial -days 3650 \
          -extfile mailpit.ext -out $out/mailpit.crt
      '';

  # Mailpit with implicit TLS on port 1465.
  mailpit =
    { pkgs, ... }:
    {
      systemd.services.mailpit = {
        wantedBy = [ "multi-user.target" ];
        serviceConfig.ExecStart = "${pkgs.mailpit}/bin/mailpit --smtp 127.0.0.1:1465 --listen 127.0.0.1:8025 --smtp-tls-cert ${certificates}/mailpit.crt --smtp-tls-key ${certificates}/mailpit.key --smtp-require-tls";
      };
    };

  trustsTestAuthority = {
    security.pki.certificateFiles = [ "${certificates}/authority.crt" ];
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
          mailpit
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
          mailpit
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
        environment.systemPackages = [ pkgs.curl ];
      };

    container =
      { pkgs, ... }:
      {
        imports = [ mailpit ];
        virtualisation.podman.enable = true;
        virtualisation.memorySize = 2048;
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
        environment.systemPackages = [ pkgs.curl ];
      };
  };

  testScript = ''
    import json

    def assert_serves_adunata(machine):
        health = json.loads(machine.succeed("curl --fail --silent http://127.0.0.1:8080/api/health"))
        assert health == {"status": "ready"}, health
        page = machine.succeed("curl --fail --silent http://127.0.0.1:8080/")
        assert "<title>Adunata</title>" in page, page

    def request_sign_in(machine):
        machine.wait_for_open_port(1465)
        machine.succeed(
            "curl --fail --silent -H 'content-type: application/json'"
            " -d '{\"mail\":\"organizer@example.org\"}' http://127.0.0.1:8080/api/sign-in-requests"
        )

    def assert_mail_arrives(machine):
        machine.wait_until_succeeds(
            "curl --fail --silent http://127.0.0.1:8025/api/v1/messages | grep --quiet '\"total\":1'", timeout=90
        )

    start_all()

    with subtest("Install on NixOS"):
        nixos.wait_for_unit("adunata.service")
        nixos.wait_for_open_port(8080)
        assert_serves_adunata(nixos)
        applied = nixos.succeed("sudo -u postgres psql -d adunata -tAc 'SELECT count(*) FROM _sqlx_migrations WHERE success'")
        assert int(applied) >= 2, applied
        exposure = nixos.succeed("systemd-analyze security adunata.service | tail -1")
        print(exposure)

    with subtest("Mail over TLS on NixOS, trusted through the host's store"):
        request_sign_in(nixos)
        assert_mail_arrives(nixos)

    with subtest("Install from release binary"):
        binary.wait_for_unit("postgresql.service")
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

    with subtest("Mail over TLS from the release binary, trusted through the host's store"):
        request_sign_in(binary)
        assert_mail_arrives(binary)

    with subtest("Install as container"):
        container.wait_for_unit("postgresql.service")
        container.succeed("podman load --input ${image}")
        container_settings = (
            " --network host"
            " --env ADNT_HTTP_LISTEN_ADDRESS=127.0.0.1:8080"
            " --env ADNT_HTTP_PUBLIC_URL=http://localhost:8080"
            " --env ADNT_DATABASE_URL=postgres://adunata@127.0.0.1/adunata"
            " --env ADNT_SMTP_HOST=127.0.0.1"
            " --env ADNT_SMTP_PORT=1465"
            " --env ADNT_SMTP_ENCRYPTION=tls"
            " --env 'ADNT_MAIL_SENDER=Adunata <adunata@example.org>'"
            " --env ADNT_LOCALIZATION_TIME_ZONE=Europe/Berlin"
            " --env ADNT_BOOTSTRAP_ORGANIZER_MAIL=organizer@example.org"
            " --env ADNT_BOOTSTRAP_ORGANIZER_FIRST_NAME=Ada"
            " --env ADNT_BOOTSTRAP_ORGANIZER_LAST_NAME=Lovelace"
        )
        container.succeed("podman run --detach --name untrusting" + container_settings + " adunata:latest")
        container.wait_for_open_port(8080)
        assert_serves_adunata(container)

    with subtest("Image ships a bundle of certificate authorities"):
        bundle = container.succeed(
            "podman image inspect adunata:latest --format '{{range .Config.Env}}{{println .}}{{end}}'"
            " | sed --quiet 's/^SSL_CERT_FILE=//p'"
        ).strip()
        assert bundle, "the image sets no SSL_CERT_FILE"
        root = container.succeed("podman image mount adunata:latest").strip()
        container.succeed(f"test -s {root}{bundle}")

    with subtest("Image rejects a mail server outside its bundle"):
        request_sign_in(container)
        container.wait_until_succeeds("podman logs untrusting 2>&1 | grep 'mail delivery failed' | grep --quiet 'encryption'", timeout=60)
        container.succeed("curl --fail --silent http://127.0.0.1:8025/api/v1/messages | grep --quiet '\"total\":0'")
        container.succeed("podman stop untrusting")

    with subtest("Image trusts the bundle that SSL_CERT_FILE names"):
        container.succeed(
            "podman run --detach --name trusting"
            " --volume ${certificates}/authority.crt:/authority.crt:ro"
            " --env SSL_CERT_FILE=/authority.crt"
            + container_settings + " adunata:latest"
        )
        assert_mail_arrives(container)

    with subtest("Stop gracefully"):
        nixos.succeed("systemctl stop adunata.service")
        nixos.succeed("systemctl show adunata.service --property=ExecMainStatus | grep -x ExecMainStatus=0")
  '';
}
