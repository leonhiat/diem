---
id: libra-framework
title: Libra Framework
custom_edit_url: https://github.com/libra/libra/edit/master/language/stdlib/README.md
---

## The Libra Framework

The Libra Framework defines the standard actions that can be performed on-chain
both by the Libra VM---through the various prologue/epilogue functions---and by
users of the blockchain---through the allowed set of transactions. This
directory contains different directories that hold the source Move
modules and transaction scripts, along with a framework for generation of
documentation, ABIs, and error information from the Move source
files. See the [Layout](#layout) section for a more detailed overview of the structure.

## Documentation

Each of the main components of the Libra Framework and contributing guidelines are documented separately. Particularly:
* Documentation for the set of allowed transaction script can be found in [transaction_scripts/doc/transaction_script_documentation.md](transaction_scripts/doc/transaction_script_documentation.md).
* The overview documentation for the Move modules can be found in [modules/doc/overview.md](modules/doc/overview.md).
* Contributing guidelines and basic coding standards for the Libra Framework can be found in [CONTRIBUTING.md](CONTRIBUTING.md).

## Compilation and Generation

Recompilation of the Libra Framework and the regeneration of the documents,
ABIs, and error information can be performed by running `cargo run` from this
directory. There are a number of different options available and these are
explained in the help for this command by running `cargo run -- --help` in this
directory. Compilation and generation will be much faster if run in release
mode (`cargo run --release`).

## Layout
The overall structure of the Libra Framework is as follows:

```
├── compiled                                # Generated files and public rust interface to the Libra Framework
│   ├── error_descriptions/*.errmap         # Generated error descriptions for use by the Move Explain tool
│   ├── src                                 # External Rust interface/library to use the Libra Framework
│   ├── stdlib                              # The compiled Move bytecode of the Libra Framework source modules
│   └── transaction_scripts                 # Generated ABIs and bytecode for each transaction script in the allowlist
│       ├── abi/*.abi                       # Directory containing generated ABIs
│       └── *.mv
├── modules                                 # Libra Framework source modules and generated documentation
│   ├── *.move
│   └── doc/*.md                            # Generated documentation for the Libra Framework modules
├── src                                     # Compilation and generation of information from Move source files in the Libra Framework. Not designed to be used as a Rust library
├── tests
└── transaction_scripts/*.move              # Move source files for allowed transaction scripts
    └── doc/*.md                            # Generated documentation for allowed transaction scripts
```
