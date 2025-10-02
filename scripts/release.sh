#!/bin/bash
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Global variables
CURRENT_VERSION=""
NEW_VERSION=""
RELEASE_TYPE=""
RELEASE_BRANCH=""

# Helper functions
error() { echo -e "${RED}❌ Error: $1${NC}" >&2; exit 1; }
success() { echo -e "${GREEN}✅ $1${NC}"; }
info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }

# Validate environment
validate_environment() {
    # Check if we're in a git repository
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        error "Not in a git repository"
    fi

    # Check if on main/master branch
    local current_branch
    current_branch=$(git branch --show-current)
    if [[ "$current_branch" != "main" && "$current_branch" != "master" ]]; then
        error "Must be on main/master branch (currently on: $current_branch)"
    fi

    # Check if working tree is clean
    if ! git diff --quiet || ! git diff --cached --quiet; then
        error "Working tree is not clean. Please commit or stash changes first."
    fi
}

# Parse and validate arguments
parse_arguments() {
    RELEASE_TYPE="${1:-}"
    if [[ "$RELEASE_TYPE" != "patch" && "$RELEASE_TYPE" != "minor" && "$RELEASE_TYPE" != "major" ]]; then
        error "Usage: $0 <patch|minor|major>"
    fi
}

# Get current version from Cargo.toml
get_current_version() {
    CURRENT_VERSION=$(grep '^version =' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
    if [[ -z "$CURRENT_VERSION" ]]; then
        error "Could not determine current version from Cargo.toml"
    fi
}

# Calculate new version
calculate_new_version() {
    IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"
    case "$RELEASE_TYPE" in
        "patch")
            NEW_PATCH=$((PATCH + 1))
            NEW_VERSION="$MAJOR.$MINOR.$NEW_PATCH"
            ;;
        "minor")
            NEW_MINOR=$((MINOR + 1))
            NEW_VERSION="$MAJOR.$NEW_MINOR.0"
            ;;
        "major")
            NEW_MAJOR=$((MAJOR + 1))
            NEW_VERSION="$NEW_MAJOR.0.0"
            ;;
    esac
}

# Display release plan and get confirmation
display_release_plan() {
    echo
    echo "=========================================="
    echo "🚀 RELEASE PLAN"
    echo "=========================================="
    echo "Current version: $CURRENT_VERSION"
    echo "New version:     $NEW_VERSION"
    echo "Release type:    $RELEASE_TYPE"
    echo
    echo "Will perform:"
    echo "  1. Create release/v$NEW_VERSION branch"
    echo "  2. Update Cargo.toml version"
    echo "  3. Update CHANGELOG.md"
    echo "  4. Run full validation (fmt, lint, test, audit, license, package)"
    echo "  5. Commit changes: 'release: bump to v$NEW_VERSION'"
    echo "  6. Merge to main branch"
    echo "  7. Create git tag v$NEW_VERSION"
    echo "  8. Push main and tag"
    echo
    echo "Release targets:"
    echo "  • Crates.io (cargo publish)"
    echo "  • GitHub Release (automated via workflow)"
    echo "=========================================="
    echo

    # Confirm with user (default NO)
    read -p "Continue with release? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        info "Release cancelled by user"
        exit 0
    fi
}

# Create release branch
create_release_branch() {
    RELEASE_BRANCH="release/v$NEW_VERSION"
    info "Creating release branch: $RELEASE_BRANCH"
    git checkout -b "$RELEASE_BRANCH"
}

# Bump version using cargo-bump
bump_version() {
    info "Bumping version to $NEW_VERSION"
    cargo bump "$RELEASE_TYPE"
}

# Verify version bump worked
verify_version_bump() {
    local updated_version
    updated_version=$(grep '^version =' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
    if [[ "$updated_version" != "$NEW_VERSION" ]]; then
        error "Version bump failed. Expected $NEW_VERSION, got $updated_version"
    fi
}

# Update CHANGELOG.md
update_changelog() {
    info "Updating CHANGELOG.md"
    if [[ -f "CHANGELOG.md" ]]; then
        # Find the "## [Unreleased]" section and move it to the new version
        sed -i.bak "s/## \[Unreleased\]/## [$NEW_VERSION] - $(date +%Y-%m-%d)/" CHANGELOG.md
        # Add new Unreleased section
        sed -i.bak '1a\
## [Unreleased]\
\
' CHANGELOG.md
        rm CHANGELOG.md.bak
        success "CHANGELOG.md updated"
    else
        warning "CHANGELOG.md not found, skipping update"
    fi
}

# Run all validation checks
run_validation() {
    info "Running pre-release validation..."

    # Format check
    info "Checking code formatting..."
    if ! cargo fmt --all --check; then
        error "Code formatting check failed"
    fi

    # Lint check
    info "Running clippy..."
    if ! cargo clippy --all-targets --all-features -- -D warnings; then
        error "Clippy check failed"
    fi

    # Test
    info "Running tests..."
    if ! cargo test --all-features; then
        error "Tests failed"
    fi

    # Doc tests
    info "Running doc tests..."
    if ! cargo test --doc --all-features; then
        error "Doc tests failed"
    fi

    # Security audit
    info "Running security audit..."
    if ! cargo audit --ignore RUSTSEC-2023-0071; then
        error "Security audit failed"
    fi

    # License check
    info "Checking license compliance..."
    if ! cargo deny check licenses || ! cargo deny check bans || ! cargo deny check advisories; then
        error "License compliance check failed"
    fi

    # Package validation
    info "Validating package..."
    if ! cargo package --allow-dirty; then
        error "Package validation failed"
    fi

    success "All validation checks passed!"
}

# Commit changes
commit_changes() {
    local commit_msg="release: bump to v$NEW_VERSION"
    info "Committing changes: $commit_msg"
    git add Cargo.toml CHANGELOG.md
    git commit -m "$commit_msg"
}

# Merge back to main and create tag
merge_and_tag() {
    # Merge back to main
    info "Merging to main branch..."
    git checkout main
    if ! git merge "$RELEASE_BRANCH" --no-ff -m "chore: merge release v$NEW_VERSION"; then
        error "Merge failed"
    fi

    # Create tag
    info "Creating git tag: v$NEW_VERSION"
    git tag -a "v$NEW_VERSION" -m "Release v$NEW_VERSION"

    # Clean up release branch
    info "Cleaning up release branch..."
    git branch -d "$RELEASE_BRANCH"
}

# Push changes
push_changes() {
    info "Pushing main and tag..."
    git push origin main
    git push origin "v$NEW_VERSION"
}

# Display success message
display_success() {
    success "🎉 Release v$NEW_VERSION completed successfully!"
    info "The release workflow will automatically publish to crates.io and create a GitHub release."
    echo
    info "Next steps:"
    echo "  • Monitor the GitHub Actions workflow"
    echo "  • Check crates.io for package availability"
    echo "  • Verify GitHub release creation"
}

# Main function
main() {
    success "Starting release process..."

    validate_environment
    parse_arguments "$@"
    get_current_version
    calculate_new_version
    display_release_plan

    create_release_branch
    bump_version
    verify_version_bump
    update_changelog
    run_validation
    commit_changes
    merge_and_tag
    push_changes
    display_success
}

# Call main function with all arguments
main "$@"

