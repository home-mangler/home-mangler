# home-mangler

`home-mangler` is a Nix Flakes home directory management tool.

`home-mangler` is configured with a Nix Flake in `~/.config/home-mangler/flake.nix`:

```nix
{
  # TODO
}
```

Wishlist:

- You can provide a list of packages to install.
  - E.g., `[ pkgs.broot pkgs.ripgrep ]`
  - I'm imagining a flake output like `home-mangler.packages` or something.
  - Think `nix profile` but good UX.
- You can provide a derivation of files to overlay in the home directory.
  - E.g., `~/your-dotfiles-dir`, potentially with `filterSource` or a more
    sophisticated tool like `rcm`. Or `chezmoi`.
    - Does Nix think home-directory-relative paths are impure?
  - Maybe you could also provide an attribute set with directory structure.
- You can provide a program or programs to run (in your home directory).
  - Like uh `home-manager switch` haha.
  - Or `topgrade`.
  - Or `rcup`.
- There's a configuration file for specifying defaults for options like:
  - Automatically update Flake inputs
- Maybe it's compatible with home-manager modules?
