# Clipboard Configuration
You can configure the way copying works in nyaa using several options. The configuration looks like:
```toml
[clipboard]
x11_selection = "..."
cmd = "..."
shell_cmd = "..."
```

For those using X11, you can change the option `x11_selection` to either `Primary` or `Clipboard` to change which selection X11 will copy the content into. This would look like:
```toml
[clipboard]
x11_selection = "Primary" # or "Clipboard"
```

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
