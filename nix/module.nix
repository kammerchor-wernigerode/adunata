package:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.adunata;
  localDatabaseUrl = "postgres:///adunata?host=/run/postgresql&user=adunata";
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

    http = {
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

      trustedProxies = lib.mkOption {
        type = lib.types.listOf lib.types.str;
        default = [
          "127.0.0.1/32"
          "::1/128"
        ];
        description = "Networks in CIDR notation of the reverse proxies whose X-Forwarded-For header Adunata trusts.";
      };
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

    localization = {
      defaultLanguage = lib.mkOption {
        type = lib.types.enum [
          "de"
          "en"
        ];
        default = "en";
        description = "Language of the first organizer and of members that grant-organizer creates.";
      };

      timeZone = lib.mkOption {
        type = lib.types.str;
        example = "Europe/Berlin";
        description = "IANA time zone of the instance. Adunata shows wall-clock times in it and compares them with the present.";
      };
    };

    retention = {
      commentDays = lib.mkOption {
        type = lib.types.either lib.types.ints.unsigned (lib.types.enum [ "never" ]);
        default = 30;
        description = "Days after the last day of a gathering until Adunata clears the comments of its responses, or never.";
      };

      responseDays = lib.mkOption {
        type = lib.types.either lib.types.ints.unsigned (lib.types.enum [ "never" ]);
        default = 365;
        description = "Days after the last day of a gathering until Adunata deletes its invitations and responses, or never.";
      };
    };

    legal = {
      privacyUrl = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        example = "https://choir.example.org/privacy";
        description = "Address of the privacy notice that every page and every invitation mail links.";
      };

      imprintUrl = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        example = "https://choir.example.org/imprint";
        description = "Address of the imprint that every page and every invitation mail links.";
      };
    };

    bootstrap = {
      organizerMail = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        description = "Mail address of the member whom Adunata makes organizer when the instance has no active organizer.";
      };

      organizerFirstName = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        description = "First name of the first organizer. Required with organizerMail.";
      };

      organizerLastName = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        description = "Last name of the first organizer. Required with organizerMail.";
      };
    };
  };

  config = lib.mkIf cfg.enable {
    assertions = [
      {
        assertion =
          cfg.bootstrap.organizerMail == null
          || (cfg.bootstrap.organizerFirstName != null && cfg.bootstrap.organizerLastName != null);
        message = "services.adunata.bootstrap.organizerMail needs organizerFirstName and organizerLastName.";
      }
    ];

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
        ADNT_HTTP_PUBLIC_URL = cfg.http.publicUrl;
        ADNT_HTTP_LISTEN_ADDRESS = cfg.http.listenAddress;
        ADNT_HTTP_TRUSTED_PROXIES = lib.concatStringsSep "," cfg.http.trustedProxies;
        ADNT_DATABASE_URL = cfg.database.url;
        ADNT_SMTP_HOST = cfg.smtp.host;
        ADNT_SMTP_PORT = toString cfg.smtp.port;
        ADNT_SMTP_ENCRYPTION = cfg.smtp.encryption;
        ADNT_MAIL_SENDER = cfg.mail.sender;
        ADNT_MAIL_MAX_ATTEMPTS = toString cfg.mail.maxAttempts;
        ADNT_LOCALIZATION_DEFAULT_LANGUAGE = cfg.localization.defaultLanguage;
        ADNT_LOCALIZATION_TIME_ZONE = cfg.localization.timeZone;
        ADNT_RETENTION_COMMENT_DAYS = toString cfg.retention.commentDays;
        ADNT_RETENTION_RESPONSE_DAYS = toString cfg.retention.responseDays;
      }
      // lib.optionalAttrs (cfg.smtp.username != null) { ADNT_SMTP_USERNAME = cfg.smtp.username; }
      // lib.optionalAttrs (cfg.legal.privacyUrl != null) {
        ADNT_LEGAL_PRIVACY_URL = cfg.legal.privacyUrl;
      }
      // lib.optionalAttrs (cfg.legal.imprintUrl != null) {
        ADNT_LEGAL_IMPRINT_URL = cfg.legal.imprintUrl;
      }
      // lib.optionalAttrs (cfg.bootstrap.organizerMail != null) {
        ADNT_BOOTSTRAP_ORGANIZER_MAIL = cfg.bootstrap.organizerMail;
        ADNT_BOOTSTRAP_ORGANIZER_FIRST_NAME = cfg.bootstrap.organizerFirstName;
        ADNT_BOOTSTRAP_ORGANIZER_LAST_NAME = cfg.bootstrap.organizerLastName;
      }
      // lib.optionalAttrs (cfg.database.passwordFile != null) {
        ADNT_DATABASE_PASSWORD_FILE = "%d/database-password";
      }
      // lib.optionalAttrs (cfg.smtp.passwordFile != null) {
        ADNT_SMTP_PASSWORD_FILE = "%d/smtp-password";
      };

      serviceConfig = {
        ExecStart = "${lib.getExe cfg.package} serve";
        Restart = "on-failure";
        RestartSec = 5;
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
