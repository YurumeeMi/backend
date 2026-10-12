devShells.${system}.default =
    pkgs.mkShell {
        packages = with.pkgs; [
            rustc
            cargo
            rust-analyzer
            postgresql
            pkg-config
        ];
    };
