package:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.adunata;
  localDatabaseUrl = "postgres:///adunata?host=/run/postgresql";
in
{
  options.services.adunata = {
    enable = lib.mkEnableOption "Adunata, a web application for choirs and instrumental ensembles";

    package = lib.mkOption {
      type = lib.types.package;
      default = package pkgs;
      defaultText = lib.literalExpression "the package of this flake";
      description = "The Adunata package to run.";
    };

    publicUrl = lib.mkOption {
      type = lib.types.str;
      example = "https://adunata.example.org";
      description = "The address under which people reach the instance. Every link in a mail starts with it.";
    };

    listenAddress = lib.mkOption {
      type = lib.types.str;
      default = "127.0.0.1:8080";
      description = "Address and port on which Adunata accepts HTTP requests. Put a reverse proxy with HTTPS in front of it.";
    };

    defaultLanguage = lib.mkOption {
      type = lib.types.enum [
        "de"
        "en"
      ];
      default = "en";
      description = "Language of mail to recipients without a stored language.";
    };

    database = {
      createLocally = lib.mkOption {
        type = lib.types.bool;
        default = true;
        description = "Whether to create a local PostgreSQL database and connect to it through its Unix socket.";
      };

      url = lib.mkOption {
        type = lib.types.str;
        default = localDatabaseUrl;
        description = "The PostgreSQL connection. Keep the password out of it and use passwordFile.";
      };

      passwordFile = lib.mkOption {
        type = lib.types.nullOr lib.types.path;
        default = null;
        description = "File that holds the database password.";
      };
    };

    mail = {
      sender = lib.mkOption {
        type = lib.types.str;
        example = "Adunata <adunata@example.org>";
        description = "The sender of every mail.";
      };

      maxAttempts = lib.mkOption {
        type = lib.types.ints.between 1 100;
        default = 8;
        description = "How often Adunata tries to deliver a mail.";
      };
    };

    smtp = {
      host = lib.mkOption {
        type = lib.types.str;
        description = "Host name of the mail server.";
      };

      port = lib.mkOption {
        type = lib.types.port;
        default = 587;
        description = "Port of the mail server.";
      };

      encryption = lib.mkOption {
        type = lib.types.enum [
          "starttls"
          "tls"
          "none"
        ];
        default = "starttls";
        description = "How Adunata protects the connection to the mail server.";
      };

      username = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        description = "User name at the mail server.";
      };

      passwordFile = lib.mkOption {
        type = lib.types.nullOr lib.types.path;
        default = null;
        description = "File that holds the password at the mail server.";
      };
    };
  };

  config = lib.mkIf cfg.enable {
    services.postgresql = lib.mkIf cfg.database.createLocally {
      enable = true;
      ensureDatabases = [ "adunata" ];
      ensureUsers = [
        {
          name = "adunata";
          ensureDBOwnership = true;
        }
      ];
    };

    systemd.services.adunata = {
      description = "Adunata";
      wantedBy = [ "multi-user.target" ];
      wants = [ "network-online.target" ];
      after = [ "network-online.target" ] ++ lib.optional cfg.database.createLocally "postgresql.service";
      requires = lib.optional cfg.database.createLocally "postgresql.service";

      environment = {
        ADUNATA_PUBLIC_URL = cfg.publicUrl;
        ADUNATA_LISTEN_ADDRESS = cfg.listenAddress;
        ADUNATA_DEFAULT_LANGUAGE = cfg.defaultLanguage;
        ADUNATA_DATABASE_URL = cfg.database.url;
        ADUNATA_SMTP_HOST = cfg.smtp.host;
        ADUNATA_SMTP_PORT = toString cfg.smtp.port;
        ADUNATA_SMTP_ENCRYPTION = cfg.smtp.encryption;
        ADUNATA_MAIL_SENDER = cfg.mail.sender;
        ADUNATA_MAIL_MAX_ATTEMPTS = toString cfg.mail.maxAttempts;
      }
      // lib.optionalAttrs (cfg.smtp.username != null) { ADUNATA_SMTP_USERNAME = cfg.smtp.username; }
      // lib.optionalAttrs (cfg.database.passwordFile != null) {
        ADUNATA_DATABASE_PASSWORD_FILE = "%d/database-password";
      }
      // lib.optionalAttrs (cfg.smtp.passwordFile != null) {
        ADUNATA_SMTP_PASSWORD_FILE = "%d/smtp-password";
      };

      serviceConfig = {
        ExecStart = lib.getExe cfg.package;
        Restart = "on-failure";
        DynamicUser = true;
        User = "adunata";
        LoadCredential =
          lib.optional (cfg.database.passwordFile != null) "database-password:${cfg.database.passwordFile}"
          ++ lib.optional (cfg.smtp.passwordFile != null) "smtp-password:${cfg.smtp.passwordFile}";
        UMask = "0077";
        NoNewPrivileges = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        PrivateTmp = true;
        PrivateDevices = true;
        ProtectClock = true;
        ProtectHostname = true;
        ProtectKernelLogs = true;
        ProtectKernelModules = true;
        ProtectKernelTunables = true;
        ProtectControlGroups = true;
        RestrictAddressFamilies = [
          "AF_INET"
          "AF_INET6"
          "AF_UNIX"
        ];
        RestrictNamespaces = true;
        RestrictRealtime = true;
        LockPersonality = true;
        MemoryDenyWriteExecute = true;
        SystemCallArchitectures = "native";
        SystemCallFilter = [
          "@system-service"
          "~@privileged @resources"
        ];
        CapabilityBoundingSet = "";
      };
    };
  };
}
