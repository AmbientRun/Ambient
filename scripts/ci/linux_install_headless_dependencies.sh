#!/bin/sh
sudo add-apt-repository ppa:oibaf/graphics-drivers -y
sudo apt-get update
sudo apt install -y libxcb-xfixes0-dev mesa-vulkan-drivers
