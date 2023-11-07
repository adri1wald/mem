# Mem

Mem is a CLI tool that lets you store memories and later retrieve them semantically.

The initial idea came from a need to store useful CLI commands that I use infrequently and therefore can't remember. I wanted to be able to store them in a way that I could later retrieve them without having to use specific keywords.

## Installation

```bash
# N.B. this will attempt to install the binary to ~/.local/bin.
# If you want to install it elsewhere, you'll need to do it manually.
$ ./scripts/build.sh
```

## Usage

```bash
# Set the OpenAI API key
$ mem set-key
# Add a memory
$ mem insert "git diff HEAD^ HEAD" "show diff between last commit and current commit"
# Get the best matched memory
$ mem get "diff between commits"
# List top k memories (default k = 10)
$ mem list "diffs"
```
