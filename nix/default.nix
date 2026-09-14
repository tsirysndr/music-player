# Entry point for building the released packages outside the flake:
#
#   nix-build nix                          # music-player CLI (the default)
#   nix-build nix -A music-player-desktop  # Slint desktop client (Linux)
#
# Each package lives in its own callPackage-style file, ready to be copied
# verbatim into a nixpkgs checkout under pkgs/by-name/mu/<pname>/package.nix.
{
  pkgs ? import <nixpkgs> { },
}:

rec {
  music-player = pkgs.callPackage ./music-player.nix { };
  music-player-desktop = pkgs.callPackage ./music-player-desktop.nix { };
  default = music-player;
}
