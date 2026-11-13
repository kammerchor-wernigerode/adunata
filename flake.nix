{
  description = "Adunata, a web application for choirs and instrumental ensembles to organize gatherings";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";

  outputs =
    { nixpkgs, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      formatter = forAllSystems (pkgs: pkgs.nixfmt-rfc-style);

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
