#!/bin/bash

# Show help message
show_help() {
    cat << EOF
Usage: $0 <package> [args...]

Run a package in the workspace using cargo run.

Arguments:
    package     The package name to run (e.g., playshelf-server, playshelf-client)
    args        Optional arguments to pass to the program (use -- to separate cargo args)

Examples:
    $0 playshelf-server
    $0 playshelf-server -- --help
    $0 playshelf-client -- --port 8080

Available packages:
    - playshelf-server
    - playshelf-client

Options:
    -h, --help    Show this help message
EOF
}

# Check for help flag
if [[ "$1" == "-h" ]] || [[ "$1" == "--help" ]]; then
    show_help
    exit 0
fi

# Set default package if not provided
if [[ -z "$1" ]]; then
    package="playshelf-server"
else
    package=$1
    shift  # Remove first argument, remaining args are for cargo run
fi

# Check if first arg is '--' separator
if [[ "$1" == "--" ]]; then
    shift  # Remove '--' separator
    program_args=("$@")
else
    # No separator, all remaining args go to the program
    program_args=("$@")
fi

# Set default args if none provided
if [[ ${#program_args[@]} -eq 0 ]]; then
    program_args=("--dev")
fi

echo "Running package: $package"
if [[ ${#program_args[@]} -gt 0 ]]; then
    echo "With arguments: ${program_args[@]}"
fi
echo ""

cargo run -p "$package" -- "${program_args[@]}"

