{
  pkgs,
  module,
  release,
  image,
}:
pkgs.testers.runNixOSTest {
  name = "adunata";

  nodes = {
    nixos =
      { pkgs, ... }:
      {
        imports = [ module ];
        services.adunata = {
          enable = true;
          publicUrl = "http://localhost:8080";
          mail.sender = "Adunata <adunata@example.org>";
          smtp = {
            host = "127.0.0.1";
            port = 1025;
            encryption = "none";
          };
        };
        systemd.services.mailpit = {
          wantedBy = [ "multi-user.target" ];
          serviceConfig.ExecStart = "${pkgs.mailpit}/bin/mailpit --smtp 127.0.0.1:1025 --listen 127.0.0.1:8025";
        };
      };

    binary =
      { pkgs, ... }:
      {
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

    start_all()

    with subtest("Install on NixOS"):
        nixos.wait_for_unit("adunata.service")
        nixos.wait_for_open_port(8080)
        assert_serves_adunata(nixos)
        applied = nixos.succeed("sudo -u postgres psql -d adunata -tAc 'SELECT count(*) FROM _sqlx_migrations WHERE success'")
        assert int(applied) >= 2, applied
        exposure = nixos.succeed("systemd-analyze security adunata.service | tail -1")
        print(exposure)

    with subtest("Install from release binary"):
        binary.wait_for_unit("postgresql.service")
        binary.succeed(
            "install -D -m 0755 ${release}/adunata /usr/local/bin/adunata",
            "mkdir -p /etc/adunata /run/systemd/system",
            "cp ${release}/adunata.service /run/systemd/system/adunata.service",
            "sed -e 's#adunata@localhost#adunata@127.0.0.1#' -e 's#mail.example.org#127.0.0.1#' -e 's#https://adunata.example.org#http://localhost:8080#' ${release}/adunata.env.example > /etc/adunata/adunata.env",
            "touch /etc/adunata/database-password /etc/adunata/smtp-password",
            "systemctl daemon-reload",
            "systemctl start adunata.service",
        )
        binary.wait_for_open_port(8080)
        assert_serves_adunata(binary)
        _, linkage = binary.execute("ldd /usr/local/bin/adunata 2>&1")
        print(linkage)
        assert "statically linked" in linkage or "not a dynamic executable" in linkage, linkage

    with subtest("Install as container"):
        container.wait_for_unit("postgresql.service")
        container.succeed("podman load --input ${image}")
        container.succeed(
            "podman run --detach --network host"
            " --env ADUNATA_LISTEN_ADDRESS=127.0.0.1:8080"
            " --env ADUNATA_PUBLIC_URL=http://localhost:8080"
            " --env ADUNATA_DATABASE_URL=postgres://adunata@127.0.0.1/adunata"
            " --env ADUNATA_SMTP_HOST=127.0.0.1"
            " --env 'ADUNATA_MAIL_SENDER=Adunata <adunata@example.org>'"
            " adunata:latest"
        )
        container.wait_for_open_port(8080)
        assert_serves_adunata(container)

    with subtest("Stop gracefully"):
        nixos.succeed("systemctl stop adunata.service")
        nixos.succeed("systemctl show adunata.service --property=ExecMainStatus | grep -x ExecMainStatus=0")
  '';
}
