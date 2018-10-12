# resolvloser prioritizes IPv6 your /etc/resolv.conf

**resolvloser** is a tool with only one goal: reordering the nameservers in
`/etc/resolv.conf` so IPv6 nameservers are listed first. It does not touch any
other configuration. The lines with the nameserver options are reused, so the
structure of the original file is preserved. A header is added so you know
something might have changed, and at which date/time.

## Build

It's rust, so get your toolchain up and running with either
[rustup](https://rustup.rs/) or the package manager on your OS.

Then, git clone and build from the root of the repo:
```
cargo build --release
```

Copy the binary to a place of your liking, for example
```bash
# cp ./target/release/resolvloser /usr/sbin/
```

## Usage

resolvloser takes a filename to use as input, or defaults to `/etc/resolv.conf`
otherwise. The reordered file will be output to stdout, or,  if `-i` is passed,
the file is modified in-place.

### Automate using systemd

Check out the examples directory for example systemd *path* and *service*
files. Put them in `/etc/systemd/system`.

The *path* unit monitors for changes in `/etc/resolv.conf`, and runs the
*service* unit on file changes. In the service file, make sure the
`ExecStart` points to the `resolvloser` binary.

```
# systemctl enable resolvloser.path
```

### Other ways

Any other place where you can create some kind of hook, you should be able to
use `resolvloser`. Other example configurations are welcome.  If you want to
alter `/etc/resolv.conf` in place, remember that you need root permissions. To
simply see what `resolvloser` does with your `resolv.conf`, run without `-i`.

```bash
$ resolvloser -h

resolvloser v0.1.0
Unrecognized option: 'h'
Usage: resolvloser RESOLVCONF_FILE [options]

Options:
    -i                  modify in-place (otherwise output on stdout)
```



## Why?

I fought resolv but could not get it to do what I wanted: I was a resolvloser.
I simply wanted to have IPv6 addresses listed first, but could not find an
option anywhere. Given that there are many different pieces of software that
alter `/etc/resolv.conf`, a simple tool to reorder afterwards seems more
pragmatic than patching all the other moving parts. And it was another excuse to
write some Rust. 
