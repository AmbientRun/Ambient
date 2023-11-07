# Screen Ray

The screen ray example demonstrates how to use `camera::screen_position_to_world_ray` and `camera::world_to_screen` on the client, and `physics::raycast` on the server.

It shows a per-player cube that can be moved around with the mouse. Clicking will spawn a new cube at the point where the mouse ray intersects with the ground.
