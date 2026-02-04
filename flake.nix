{
  description = "Fast keyring to environment variable loader";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        keyring-env = pkgs.rustPlatform.buildRustPackage {
          pname = "keyring-env";
          version = "0.1.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = with pkgs; [
            dbus
          ];

          meta = with pkgs.lib; {
            description = "Fast keyring to environment variable loader";
            homepage = "https://github.com/atomic-235/keepass_env";
            license = licenses.mit;
            platforms = platforms.linux;
            mainProgram = "keyring-env";
          };
        };

      in
      {
        packages = {
          default = keyring-env;
          keyring-env = keyring-env;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            pkg-config
            dbus
          ];
        };
      });
}
