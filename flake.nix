{
  description = "Rust dev shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    jailed-agents.url = "github:btmxh/jailed-agents";
    systems.url = "github:nix-systems/default";
    git-hooks.url = "github:cachix/git-hooks.nix";
  };

  outputs =
    {
      self,
      nixpkgs,
      jailed-agents,
      systems,
      git-hooks,
      ...
    }:
    let
      forEachSystem = nixpkgs.lib.genAttrs (import systems);
    in
    {
      checks = forEachSystem (system: {
        pre-commit-check = git-hooks.lib.${system}.run {
          src = ./.;
          hooks = {
            nixfmt.enable = true;
            statix.enable = true;
            check-yaml.enable = true;
            end-of-file-fixer.enable = true;
            trim-trailing-whitespace.enable = true;
            rustfmt.enable = true;
          };
        };
      });

      devShells = forEachSystem (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
          inherit (self.checks.${system}.pre-commit-check) shellHook enabledPackages;
        in
        {
          default = pkgs.mkShell {
            inherit shellHook;
            buildInputs = enabledPackages;

            packages =
              with pkgs;
              [
                rustc
                cargo
                rustfmt
                clippy
                pnpm
                nodejs
                bacon
                lld
                wasm-bindgen-cli

                # nix stuff
                nixfmt
                nixd
              ]
              ++ (builtins.attrValues (
                jailed-agents.lib.${system}.makeJailedAgents {
                  extraPkgs = with pkgs; [
                    rustc
                    cargo
                    rustfmt
                    clippy
                    wasm-bindgen-cli
                  ];
                }
              ));
          };
        }
      );

    };
}
