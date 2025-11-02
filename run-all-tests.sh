#!/bin/bash
# Worknest Complete Test Suite Runner
# Runs all backend and frontend tests with full automation

set -e  # Exit on first error

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Worknest Test Suite - Full Automation${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Track test results
TOTAL_TESTS=0
FAILED_TESTS=0

# Function to run tests and count results
run_test_suite() {
    local name=$1
    local command=$2

    echo -e "${BLUE}Running $name...${NC}"

    if output=$(eval "$command" 2>&1); then
        # Extract test count from "test result: ok. X passed"
        count=$(echo "$output" | grep -oP '\d+(?= passed)' | head -1)
        if [ -n "$count" ]; then
            TOTAL_TESTS=$((TOTAL_TESTS + count))
            echo -e "${GREEN}✓ $name: $count tests passed${NC}"
        else
            echo -e "${GREEN}✓ $name: completed${NC}"
        fi
        echo ""
        return 0
    else
        echo -e "${RED}✗ $name: FAILED${NC}"
        echo "$output"
        echo ""
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi
}

# Backend Tests
echo -e "${BLUE}=== Backend Tests ===${NC}"
run_test_suite "Core Models" "cargo test --package worknest-core --lib"
run_test_suite "Database Layer" "cargo test --package worknest-db --lib"
run_test_suite "Authentication" "cargo test --package worknest-auth --lib"

# Frontend Tests (WASM)
echo -e "${BLUE}=== Frontend Tests (WASM) ===${NC}"

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo -e "${RED}wasm-pack not found. Installing...${NC}"
    cargo install wasm-pack
fi

# Check if Firefox is available for headless testing
if ! command -v firefox &> /dev/null; then
    echo -e "${RED}Warning: Firefox not found. WASM tests require a browser.${NC}"
    echo -e "${RED}Install Firefox or use Chrome with appropriate flags.${NC}"
    exit 1
fi

run_test_suite "GUI State Tests" "wasm-pack test --headless --firefox crates/worknest-gui --test state_tests"
run_test_suite "GUI UI Tests" "wasm-pack test --headless --firefox crates/worknest-gui --test ui_tests"
run_test_suite "GUI E2E Tests" "wasm-pack test --headless --firefox crates/worknest-gui --test e2e_tests"

# Summary
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Test Suite Summary${NC}"
echo -e "${BLUE}========================================${NC}"
echo -e "Total Tests Passed: ${GREEN}${TOTAL_TESTS}${NC}"

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "Status: ${GREEN}ALL TESTS PASSED ✓${NC}"
    echo ""
    echo -e "${GREEN}🎉 Worknest is fully tested and ready!${NC}"
    exit 0
else
    echo -e "Failed Test Suites: ${RED}${FAILED_TESTS}${NC}"
    echo -e "Status: ${RED}TESTS FAILED ✗${NC}"
    exit 1
fi
