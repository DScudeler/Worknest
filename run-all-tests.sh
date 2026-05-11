#!/bin/bash
# Worknest Complete Test Suite Runner
# Runs all backend tests and the web frontend's typecheck/lint/build.

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Worknest Test Suite${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

TOTAL_TESTS=0
FAILED_TESTS=0

run_test_suite() {
    local name=$1
    local command=$2

    echo -e "${BLUE}Running $name...${NC}"

    if output=$(eval "$command" 2>&1); then
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

echo -e "${BLUE}=== Backend Tests ===${NC}"
run_test_suite "Core Models" "cargo test --package worknest-core --lib"
run_test_suite "Database Layer" "cargo test --package worknest-db --lib"
run_test_suite "Authentication" "cargo test --package worknest-auth --lib"
run_test_suite "API" "cargo test --package worknest-api --lib"
run_test_suite "Doctests" "cargo test --doc --workspace"

echo -e "${BLUE}=== Frontend Checks (web/) ===${NC}"
if [ -d web/node_modules ]; then
    :
else
    echo -e "${BLUE}Installing web dependencies (one-time)...${NC}"
    (cd web && npm ci) || (cd web && npm install)
fi
run_test_suite "Web Typecheck" "cd web && npm run typecheck"
run_test_suite "Web Lint" "cd web && npm run lint"
run_test_suite "Web Build" "cd web && npm run build"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Test Suite Summary${NC}"
echo -e "${BLUE}========================================${NC}"
echo -e "Total Backend Tests Passed: ${GREEN}${TOTAL_TESTS}${NC}"

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "Status: ${GREEN}ALL CHECKS PASSED ✓${NC}"
    exit 0
else
    echo -e "Failed Suites: ${RED}${FAILED_TESTS}${NC}"
    echo -e "Status: ${RED}CHECKS FAILED ✗${NC}"
    exit 1
fi
