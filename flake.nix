{
  description = "Rust dev shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    jailed-agents.url = "github:btmxh/jailed-agents";
  };

  outputs =
    {
      nixpkgs,
      jailed-agents,
      ...
    }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
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
          ++ (
            builtins.attrValues (
              jailed-agents.lib.${system}.makeJailedAgents {
                extraPkgs = with pkgs; [
                  rustc
                  cargo
                  rustfmt
                  clippy
                  wasm-bindgen-cli
                ];
              }
            )
          );
      };
    };
}
