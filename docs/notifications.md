# Notifications
Notifications will pop up when certain events happen, like errors or source changes. Errors will persist until dismissed, while normal notifications will go away after some duration.

The default configuration for notifications looks like:
```toml
# Your config in ~/.config/nyaa/config.toml or %appdata%\nyaa\config\config.toml
# ...

[notifications]
position = "TopRight"
duration = 3.0
max_width = 75
animation_speed = 4.0

# ...
```

`position` refers to which corner the notifications will show up. Possible values are `TopLeft`, `TopRight`, `BottomLeft`, and `BottomRight`.

`duration` refers to how long notifications will persist (measured in seconds, not including time taken for animation).

`max_width` refers to the maximum width (in columns) that notifications can be.

`animation_speed` refers to how fast notifications will animate (low values indicate slow animation, high values indicate fast animation).
