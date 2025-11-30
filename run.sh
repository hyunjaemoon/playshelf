#!/bin/bash

# Cleanup function to kill background processes
cleanup() {
    echo ""
    echo "Shutting down..."
    if [[ -n $SERVER_PID ]]; then
        echo "Stopping server (PID: $SERVER_PID)..."
        kill $SERVER_PID 2>/dev/null
        wait $SERVER_PID 2>/dev/null
    fi
    if [[ -n $CLIENT_PID ]]; then
        echo "Stopping client (PID: $CLIENT_PID)..."
        kill $CLIENT_PID 2>/dev/null
        wait $CLIENT_PID 2>/dev/null
    fi
    exit 0
}

# Set up signal handlers
trap cleanup SIGINT SIGTERM

# Show help message
show_help() {
    cat << EOF
Usage: $0 [package] [args...]

Run a package in the workspace using cargo run, or run the full app (server + client).

Arguments:
    package     The package name to run (e.g., playshelf-server, playshelf-client)
                If omitted or set to "full" or "all", runs both server and client
    args        Optional arguments to pass to the program (use -- to separate cargo args)

Examples:
    $0                          # Run full app (server + client)
    $0 full                     # Run full app (server + client)
    $0 playshelf-server         # Run only server
    $0 playshelf-server -- --help
    $0 playshelf-client         # Run only client

Available packages:
    - playshelf-server
    - playshelf-client
    - full, all                 # Run both server and client

Options:
    -h, --help    Show this help message
EOF
}

# Check for help flag
if [[ "$1" == "-h" ]] || [[ "$1" == "--help" ]]; then
    show_help
    exit 0
fi

# Check if we should run the full app
if [[ -z "$1" ]] || [[ "$1" == "full" ]] || [[ "$1" == "all" ]]; then
    echo "🚀 Starting full app (server + client)..."
    echo ""
    
    # Start server in background
    echo "📡 Starting server on http://localhost:8081..."
    cargo run -p playshelf-server > /tmp/playshelf-server.log 2>&1 &
    SERVER_PID=$!
    
    # Wait a bit for server to start
    sleep 3
    
    # Check if server is still running (if not, it might have crashed)
    if ! kill -0 $SERVER_PID 2>/dev/null; then
        echo "❌ Server failed to start. Check /tmp/playshelf-server.log for details"
        cat /tmp/playshelf-server.log
        exit 1
    fi
    
    echo "✅ Server started (PID: $SERVER_PID)"
    echo "📝 Server logs: tail -f /tmp/playshelf-server.log"
    echo ""
    
    # Start client in foreground (so user can see output and interact)
    echo "🎮 Starting client..."
    echo "Press Ctrl+C to stop both server and client"
    echo ""
    
    # Check if dx command is available
    if command -v dx &> /dev/null; then
        cd playshelf-client
        # Run dx serve in foreground - it will handle its own output
        dx serve
        cd ..
    else
        # Run cargo in foreground
        cargo run -p playshelf-client
    fi
    
    # If we get here, client has exited, so cleanup
    cleanup
fi

# Single package mode
package=$1
shift  # Remove first argument, remaining args are for cargo run

# Check if first arg is '--' separator
if [[ "$1" == "--" ]]; then
    shift  # Remove '--' separator
    program_args=("$@")
else
    # No separator, all remaining args go to the program
    program_args=("$@")
fi

echo "Running package: $package"
if [[ ${#program_args[@]} -gt 0 ]]; then
    echo "With arguments: ${program_args[@]}"
fi
echo ""

cargo run -p "$package" -- "${program_args[@]}"

