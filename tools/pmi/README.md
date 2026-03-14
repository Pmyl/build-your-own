# How to use on fresh OS
### Move to projects/repositories folder
```sh
mkdir ~/Projects
cd ~/Projects
```

### Optional: Clone this repo
```sh
git clone git@github.com:Pmyl/build-your-own.git
```

### Install Rust
```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Install pmi with the necessary features (read Cargo.toml)

From github...
```sh
cargo install --git git@github.com:Pmyl/build-your-own.git pmi --no-default-features --features ubuntu
```
...or from cloned repository
```sh
cargo install --path ./build-your-own/tools/pmi --no-default-features --features ubuntu
```

> **If this step errors with `linker 'cc' not found` then search on google and install it.** \
In ubuntu: `sudo apt install build-essential` \
In fedora: `sudo dnf install gcc`

### Install stow using pmi
```sh
pmi stow --no-save
```

### Clone dot files (that will include .pmi folder)
```sh
git clone git@github.com:Pmyl/.dotfiles.git
```

### Use stow to link dotfiles (trick to be able to overwrite existing configuration)
```sh
stow -t ~ --adopt .dotfiles
git -C .dotfiles reset --hard
```

### Check if the list of applications is there
```sh
pmi
```
This should show the list of applications from `~/.pmi/applications`

### Install list of applications
```sh
pmi --all
```
