{
  description = "An executable specification for the Escaped-RDFa Namespace.";

  inputs = {
    nixpkgs.url = "github:meta-introspector/nixpkgs?ref=feature/CRQ-016-nixify";
    flake-utils.url = "github:meta-introspector/flake-utils?ref=feature/CRQ-016-nixify";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        mdbook-pkg = pkgs.mdbook;
      in
      {
        packages = {
          # The static site documentation
          default = pkgs.stdenv.mkDerivation {
            name = "namespace-spec-site";
            src = ./.;
            nativeBuildInputs = [ mdbook-pkg ];
            buildPhase = ''
              mdbook build
            '';
            installPhase = ''
              mkdir -p $out
              cp -r book/* $out/
            '';
          };

          # The machine-readable "smart contract"
          smart-contract = pkgs.runCommand "smart-contract.json" {} ''
            echo '{ "version": "1.0", "spec": "..." }' > $out
          '';
        };
      }
    );
}
