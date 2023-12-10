# tcp_locahost #
A Rust-Based TCP Server to Host Local Files on Your Internet Browser and Manage Manufacturing Report Templates

## Goals ##

This binary serves as a utility to ship with the [Fusion-360-Report-Viewer](https://github.com/v-whitetail/Fusion-360-Report-Viewer).

The scope of this project is currently limited to hosting a local directory that links to important documents.

These links are managed through a live Home Page document.

This binary anticipates the Home Page as an index.html, a Templates directory, a Reports directory, and a Resources directory.

On startup, it will check the working directory to create these components if it does not find any.

## Function ##

This binary is intended to be called as an embedded CLI. 

It expects 3 arguments:

1. The IPv4 address to serve

2. The Port to serve

3. The working direcotry to host

Currently, this binary does not have a shutdown sequence, and it will run until the process is interrupted by the application that launches it.

Within the Report Viewer, this is done by terminating the PID loop it runs on.

To that end, if you intend to change the name of this binary, ensure you also change the criteria of its termination to match.
