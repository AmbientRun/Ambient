# Input Lag Probe

This package will make client send an occasional message to the server, server will then store the timestamp of the message and the reported lag, allowing the client to calculate the actual lag (from message being sent to its effect being replicated back to it).

The measured lag is stored in resources entity in `local_lag` component and `input_lag` component for each player entity.
