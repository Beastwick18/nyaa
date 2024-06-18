# Clipboard Configuration
You can configure the way copying works in nyaa using several options. The configuration looks like:
```toml
[clipboard]
osc52 = true
selection = "..."
cmd = "..."
shell_cmd = "..."
```

They are prioritized in the order `cmd` $\rightarrow$ `osc52` $\rightarrow$ `selection`

## OSC52
By default, `cmd` is undefined so nyaa uses [OSC52](https://www.reddit.com/r/vim/comments/k1ydpn/a_guide_on_how_to_copy_text_from_anywhere/) to copy to the clipboard. It's a type of ANSI escape sequence supported by *most* terminals, and should just work. The state of support for some popular terminals are:

| Terminal | OSC52 support |
|----------|:-------------:|
| [alacritty](https://github.com/alacritty/alacritty) | **yes** |
| [contour](https://github.com/contour-terminal/contour) | **yes** |
| [far2l](https://github.com/elfmz/far2l) | **yes** |
| [foot](https://codeberg.org/dnkl/foot) | **yes** |
| [gnome terminal](https://github.com/GNOME/gnome-terminal) (and other VTE-based terminals) | [not yet](https://gitlab.gnome.org/GNOME/vte/-/issues/2495) |
| [hterm](https://chromium.googlesource.com/apps/libapps/+/master/README.md) | [**yes**](https://chromium.googlesource.com/apps/libapps/+/master/nassh/doc/FAQ.md#Is-OSC-52-aka-clipboard-operations_supported) |
| [iterm2](https://iterm2.com/) | **yes** |
| [kitty](https://github.com/kovidgoyal/kitty) | **yes** |
| [konsole](https://konsole.kde.org/) | [not yet](https://bugs.kde.org/show_bug.cgi?id=372116) |
| [qterminal](https://github.com/lxqt/qterminal#readme) | [not yet](https://github.com/lxqt/qterminal/issues/839)
| [rxvt](http://rxvt.sourceforge.net/) | **yes** |
| [st](https://st.suckless.org/) | **yes** (but needs to be enabled, see [here](https://git.suckless.org/st/commit/a2a704492b9f4d2408d180f7aeeacf4c789a1d67.html)) |
| [terminal.app](https://en.wikipedia.org/wiki/Terminal_(macOS)) | no, but see [workaround](https://github.com/roy2220/osc52pty) |
| [tmux](https://github.com/tmux/tmux) | **yes** |
| [urxvt](http://software.schmorp.de/pkg/rxvt-unicode.html) | **yes** (with a script, see [here](https://github.com/ojroques/vim-oscyank/issues/4)) |
| [wezterm](https://github.com/wez/wezterm) | [**yes**](https://wezfurlong.org/wezterm/escape-sequences.html#operating-system-command-sequences) |
| [windows terminal](https://github.com/microsoft/terminal) | **yes** |
| [xterm.js](https://xtermjs.org/) (Hyper terminal) | **yes** |
| [zellij](https://github.com/zellij-org/zellij/) | **yes** |
<sub>Source: [vim-oscyank](https://github.com/ojroques/vim-oscyank)</sub>

If your terminal is not supported, you should disable OSC52 by adding
```toml
[clipboard]
osc52 = false
```
to your `config.toml`. This will make nyaa use an alternative method for copying.

## X11/Wayland Selection
For those using X11, you can change the option `x11_selection` to either `Clipboard`, `Primary`, or `Secondary` to change which selection X11 will copy the content into. This would look like:
```toml
[clipboard]
selection = "Primary"
```

## cmd
For those using WSL, tmux, or any other non-standard clipboard manager, you can invoke a command on copy using the `cmd` and `shell_cmd` option. The placeholder `{content}` is used for substituting the text to copy into the command. `shell_cmd` is optional when running a command, and only specifies which shell to run the command with. By default it is `sh -c` for linux and `powershell.exe -Command` for windows.

For copying on WSL, this config would look like:
```toml
[clipboard]
cmd = 'echo "{content}" | clip.exe'
# shell_cmd = "bash -c" # optional
```
For tmux, a possible configuration would look like:
```toml
[clipboard]
cmd = 'tmux set-buffer "{content}"'
# shell_cmd = "bash -c" # optional
```
