"""P2-Q pilot tools (GOAL P2-Q): Python standard library only.

The same files run on the Mac, in a linux/amd64 Docker container and in the
one-off CI job; they talk to an installed `rootmark` only through its command
line, MCP over stdio and HTTP (never the `rootmark_core` binding: what is
measured is the released binary).
"""
