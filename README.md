# Dabao Tester

The dabao tester uses a Baochip to test dabaos. In particular, the baochip runs a BIO
application that checks for pin toggling on the respective Baochip DUT.

## DUT programs - Building apps loaded onto the production devices

Compiled out of this project's `xous-core` submodule and signed with beta key (can use the baosign.ps1 script directly in the directory if the credentials directory is set up, e.g. `.\baosign.ps1 -Config baremetal -Target bunnie@xx.xx.xx.xx:`):

`cargo xtask bao1x-baremetal-dabao --loader-feature dabao-selftest --git-describe v0.10.0`

Compiled out of the mainline `xous-core` `main` branch directly & signed with beta key:

`cargo xtask dabao dabao-console --no-timestamp --kernel-feature debug-proc`

The artifacts from these builds need to be copied into `code/testjig/images/` on the tester:
namely, baremetal.uf2, xous.uf2, apps.uf2, and loader.uf2

These should be signed with the 'beta' key - to prevent putting the dabao into developer
mode - but still allow for beta-key revocation on locked down devices to reject these less-reviewed
images.

## Building the LOCAL Baochip images (these run on the test jig baochip-1x itself)

Run once:

`cargo install xous-tools`

The build is fully scripted with `build.ps1`.

Manual app creation:

`cargo build --release --target riscv32imac-unknown-xous-elf --features board-dabao --features bao1x --features utralib/bao1x`
`xous-app-uf2 --elf target/riscv32imac-unknown-xous-elf/release/dabao-tester-app`

This will create a file called `apps.uf2` which you can then copy into a Dabao.

In `src/c`:
`python3 -m ziglang build "-Dmodule=dabao_tester"`

In `xous-core`:

Note that the purpose of the xous-core repo is to fix a bootloader, kernel, and loader configuration
that has a custom USB ID in it so we can tell it apart from the DUT. Thus this is built stand-alone,
and not linked against the actual app itself.

`cargo xtask bao1x-boot1`
`cargo xtask dabao dabao-console --no-timestamp --kernel-feature debug-proc --git-describe v0.10.0`

There is a convenience script called `build.ps1` that does all of this, checked into this repo.

## Submodules

This project has a submodule. It was created using this command:
`git submodule add --depth 1 https://github.com/betrusted-io/xous-core.git xous-core`

To clone the repo fresh:
`git submodule update --init --depth 1 --no-recurse-submodules xous-core`

# Setting up the Tester

The tester is built on a Raspberry Pi 4B+. For production devices, 1GiB RAM is
sufficient, but if running vscode extensions 2GiB RAM is recommended.

Pi base uses debian trixie "lite" image:

```
lsb_release -a
No LSB modules are available.
Distributor ID: Debian
Description:    Debian GNU/Linux 13 (trixie)
Release:        13
Codename:       trixie
```

`raspi-config` notes:

- enable I2C
- enable serial port, with no login prompt

`apt` install:

`sudo apt-get install emacs-nox git gpiod screen python3-dev`

Setup udev:

copy over /etc/udev/rules.d/99-com.rules

`udevadm control --reload-rules`
`udevadm trigger`

## Tailscale

Install tailscale: `curl -fsSL https://tailscale.com/install.sh | sh`
`sudo tailscale up`
Then visit the link given on a browser with the tailscale admin console activated

## venvs

User venv:

python3 -m venv ~/.venv

add

```
if [ -d "$HOME/.venv" ]; then
    source "$HOME/.venv/bin/activate"
fi
```

To ~/.bashrc

Do the same for root.

## Python installs

In su environment:

`python3 -m pip install RPi.GPIO pyudev pyserial smbus2 luma.oled`

Repeat in user environment if doing script development.

## Harden for offline operation

```
sudo apt install fake-hwclock
sudo systemctl enable fake-hwclock
sudo timedatectl set-ntp false
```

Turn of USB auto-suspend in /boot/firmware/cmdline.txt:

`console=tty1 root=PARTUUID=e8aea68e-02 rootfstype=ext4 fsck.repair=yes rootwait cfg80211.ieee80211_regdom=US usbcore.autosuspend=-1`

(add `usbcore.autosuspend=-1` with *no newline*, if it's on a different line when reading this that's just word autowrapping happening in the text renderer)