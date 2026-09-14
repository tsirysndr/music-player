{
  pkgs ? import <nixpkgs> { },
}:

rec {
  music-player = pkgs.callPackage ./music-player.nix { };
  music-player-desktop = pkgs.callPackage ./music-player-desktop.nix { };
  default = music-player;
}
