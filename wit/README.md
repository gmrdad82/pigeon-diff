<!-- This page owns the rules for the mirrored WIT worlds. -->
# WIT interface mirror

`world.wit` is this plugin's own world. `deps/host/` mirrors the host
toolbox's `pito:host` package (plughost v0.17.0) and `deps/pigeon/` the
Done desk's `pito:pigeon` package byte for byte. Change an interface in
its owning repository, never in these copies; re-copy them on a tag bump.
