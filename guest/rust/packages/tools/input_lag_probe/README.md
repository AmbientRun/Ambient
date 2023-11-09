# Input Lag Probe

This package will make client send an occasional message to the server, server will then store the timestamp of the message and the reported lag, allowing the client to calculate the actual lag (from message being sent to its effect being replicated back to it).

## Usage

Include as dependency in your Ambient package. The measured lag is stored in resources entity in `local_lag` component and `input_lag` component for each player entity.

You can use the stored values directly or show the built-in UI by sending a `ShowInputLagWindow` message to this package:
```rust
packages::input_lag_probe::messages::ShowInputLagWindow {}.send_local(packages::input_lag_probe::entity());
```
Note: the package has to be fully loaded for this to work.

This will show a window with the lag for each player.

## Configuration

- `input_frequency` resource component holds the duration between messages sent by the client. Default is 1 second.
- `smoothing_factor` resource component holds the exponential smoothing factor for the lag. Lower the value if you want quicker but less smooth updates, 1 = use the most recent value without smoothing. Default is 8.
