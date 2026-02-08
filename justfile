##################################################
# Variables
#

rust_env := "rustup show"
rust_edition := "2024"
open := if os() == "linux" { "xdg-open" } else if os() == "macos" { "open" } else { "start \"\" /max" }
app_name := "dithers"
crate_name := "dithers"
args := ""
project_directory := justfile_directory()
release := `git describe --tags --always`
version := "0.1.0"
url := "https://github.com/tschinz/dithers"

##################################################
# COMMANDS
#

# List all commands
@default:
    just --list

# Information about the environment
@info:
    echo "Environment Informations\n------------------------\n"
    echo "OS   : {{ os() }}({{ arch() }})"
    echo "Open : {{ open }}"
    echo "Rust :"
    echo "`{{ rust_env }}`"

# Install dependencies
install:
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    cargo install --locked trunk
    cargo install cargo-sbom
    brew install trivy

# install the release version (default is the latest)
install-release release=release:
    cargo install --git {{ url }} --tag {{ release }}

# install the nightly release
install-nightly:
    cargo install --git {{ url }}

# Build and copy the release version of the program
build:
    cargo build --release
    mkdir -p bin && cp target/release/{{ app_name }} bin/

# Run the program in debug mode
run args=args:
    cargo run -- {{ args }}

# create a release version of the program
changelog version=version:
    git cliff --tag {{ version }}

# Test the program in debug mode in folder test

# Test the program with comprehensive Rust tests
test:
    cargo test

# Run integration tests only
test-integration:
    cargo test --test dither_integration_tests

# Run unit tests only
test-unit:
    cargo test --lib

# Generate sample images for all algorithms (optional visual testing)
generate-samples:
    @echo "Generating samples for all dithering algorithms..."

    # Error diffusion algorithms
    cargo run -- -i test/in/glace-1280_853.jpg -o test/out/sample-floyd-steinberg-mono.jpg -d floyd-steinberg -c monochrome
    cargo run -- -i test/in/glace-1280_853.jpg -o test/out/sample-floyd-steinberg-8c.jpg -d floyd-steinberg -c color8
    cargo run -- -i test/in/glace-1280_853.jpg -o test/out/sample-floyd-steinberg-16c.jpg -d floyd-steinberg -c color16

    cargo run -- -i test/in/glace-1280_853.jpg -o test/out/sample-jarvis-mono.jpg -d jarvis -c monochrome
    cargo run -- -i test/in/glace-1280_853.jpg -o test/out/sample-jarvis-8c.jpg -d jarvis -c color8

    cargo run -- -i test/in/glace-1280_853.jpg -o test/out/sample-atkinson-mono.jpg -d atkinson -c monochrome
    cargo run -- -i test/in/glace-1280_853.jpg -o test/out/sample-atkinson-8c.jpg -d atkinson -c color8

    cargo run -- -i test/in/glace-1280_853.jpg -o test/out/sample-stucki-mono.jpg -d stucki -c monochrome
    cargo run -- -i test/in/glace-1280_853.jpg -o test/out/sample-stucki-16c.jpg -d stucki -c color16

    cargo run -- -i test/in/glace-1280_853.jpg -o test/out/sample-burkes-8c.jpg -d burkes -c color8

    cargo run -- -i test/in/glace-1280_853.jpg -o test/out/sample-sierra-mono.jpg -d sierra -c monochrome
    cargo run -- -i test/in/glace-1280_853.jpg -o test/out/sample-sierra-16c.jpg -d sierra -c color16

    cargo run -- -i test/in/glace-1280_853.jpg -o test/out/sample-two-row-sierra-8c.jpg -d two-row-sierra -c color8

    cargo run -- -i test/in/glace-1280_853.jpg -o test/out/sample-sierra-lite-mono.jpg -d sierra-lite -c monochrome

    cargo run -- -i test/in/glace-1280_853.jpg -o test/out/sample-simple2d-8c.jpg -d simple2-d -c color8

    # Ordered dithering (Bayer matrices)
    cargo run -- -i test/in/glace-1280_853.jpg -o test/out/sample-bayer2x2-mono.jpg -d bayer2x2 -c monochrome
    cargo run -- -i test/in/glace-1280_853.jpg -o test/out/sample-bayer2x2-8c.jpg -d bayer2x2 -c color8

    cargo run -- -i test/in/glace-1280_853.jpg -o test/out/sample-bayer4x4-mono.jpg -d bayer4x4 -c monochrome
    cargo run -- -i test/in/glace-1280_853.jpg -o test/out/sample-bayer4x4-16c.jpg -d bayer4x4 -c color16

    cargo run -- -i test/in/glace-1280_853.jpg -o test/out/sample-bayer8x8-mono.jpg -d bayer8x8 -c monochrome
    cargo run -- -i test/in/glace-1280_853.jpg -o test/out/sample-bayer8x8-8c.jpg -d bayer8x8 -c color8

    # No dithering (palette quantization only)
    cargo run -- -i test/in/glace-1280_853.jpg -o test/out/sample-none-mono.jpg -d none -c monochrome
    cargo run -- -i test/in/glace-1280_853.jpg -o test/out/sample-none-8c.jpg -d none -c color8

    @echo "✓ Sample images generated in test/out/"
    @echo "Generated $(ls test/out/sample-*.jpg | wc -l | tr -d ' ') sample images covering all algorithms"

# Clean up generated sample files
clean-samples:
    rm -f test/out/sample-*.jpg

# Check code with clippy
clippy:
    cargo clippy -- -D warnings

# Run rustfmt with custom configuration
rustfmt:
    find {{ invocation_directory() }} -name \*.rs -exec rustfmt {} \;

# Generate and open rustdoc documentation
doc:
    @echo "Generating rustdoc documentation..."
    cargo doc --no-deps --document-private-items
    @echo "✓ Documentation generated"
    @echo "Opening documentation in browser..."
    {{ open }} target/doc/{{ crate_name }}/index.html

# Generate rustdoc documentation without opening
doc-build:
    @echo "Generating rustdoc documentation..."
    cargo doc --no-deps --document-private-items
    @echo "✓ Documentation generated at target/doc/{{ crate_name }}/index.html"

# Generate SBOM for Dependency Track
sbom:
    cargo sbom --output-format cyclone_dx_json_1_6 > target/sbom-cyclone_dx_1_6.json

# Upload SBOM to Dependency Track (requires DT_API_KEY, DT_PROJECT_UUID, DT_BASE_URL env vars)
sbom-upload:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Uploading SBOM to Dependency Track..."
    # Load .env file if it exists
    if [[ -f .env ]]; then
        echo "Loading configuration from .env file..."
        export $(grep -v '^#' .env | grep -v '^$' | xargs)
    fi
    if [[ -z "${DT_API_KEY:-}" ]] || [[ -z "${DT_PROJECT_UUID:-}" ]] || [[ -z "${DT_BASE_URL:-}" ]]; then
        echo "Error: Required environment variables not set:"
        echo "  DT_API_KEY - Your Dependency Track API key"
        echo "  DT_PROJECT_UUID - Your project UUID"
        echo "  DT_BASE_URL - Your Dependency Track base URL"
        echo ""
        echo "Example:"
        echo "  export DT_BASE_URL=https://dt-api.zahno.dev"
        echo "  export DT_API_KEY=your_api_key_here"
        echo "  export DT_PROJECT_UUID=your_project_uuid_here"
        echo "  just sbom-upload"
        exit 1
    fi
    just sbom
    curl -X POST "${DT_BASE_URL}/api/v1/bom" \
        -H "X-Api-Key: ${DT_API_KEY}" \
        -H "Content-Type: multipart/form-data" \
        -F "project=${DT_PROJECT_UUID}" \
        -F "bom=@target/sbom-cyclone_dx_1_6.json"
    echo "✓ SBOM uploaded successfully to Dependency Track"

# Clean security reports
clean-security:
    rm -f security-report.json trivy-results.sarif

# Trivy comprehensive security scan (alias for backwards compatibility)
trivy:
    trivy fs --scanners vuln,secret,misconfig --format table .

# output the help information
help:
    cargo run -- -h
