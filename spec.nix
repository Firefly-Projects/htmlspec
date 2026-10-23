# This project's identity and the toolchain it expects.
#
# `htmlspec archive` reads `version` and refuses unless it is ordered after the
# last archived version. Advancing it is your work, or your agent's; the program
# only compares. Nix rather than another format because `htmlspec` is a flake
# input with a hash, which is a pin the build already knows how to resolve.
{
  name = "html-spec";
  version = "1.1.0";
  # The toolchain this project expects. Replace `null` with a pinned flake
  # reference to hold every machine to one binary.
  htmlspec = null;
}
