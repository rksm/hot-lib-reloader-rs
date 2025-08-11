#!/usr/bin/env python

import argparse
from pathlib import Path
from typing import Mapping


def find_cargo_toml_files(root_dir: Path = Path('.')) -> list[Path]:
    """Find all Cargo.toml files recursively from the root directory."""
    return list(root_dir.rglob('Cargo.toml'))


def is_workspace_cargo_toml(cargo_toml_path: Path) -> bool:
    """Check if a Cargo.toml file contains a [workspace] section."""
    try:
        with open(cargo_toml_path, 'r') as f:
            content = f.read()
            return '[workspace]' in content
    except Exception:
        return False


class RepositoryCrates:
    workspace_dirs: list[Path]
    standalone_crates: list[Path]
    workspace_crates: Mapping[Path, list[Path]]

    def __init__(self) -> None:
        # Find all Cargo.toml files
        cargo_tomls = find_cargo_toml_files()

        # Identify workspace directories
        workspace_dirs = []
        for toml in cargo_tomls:
            if is_workspace_cargo_toml(toml):
                workspace_dirs.append(toml.parent)

        # Sort workspace dirs by depth (deepest first) to find the closest parent
        workspace_dirs.sort(key=lambda p: len(p.parts), reverse=True)

        # Get all crate directories (parent dirs of Cargo.toml files)
        crate_dirs = [toml.parent for toml in cargo_tomls]

        standalone_crates: list[Path] = []
        workspace_crates: dict[Path, list[Path]] = {}

        for crate_dir in crate_dirs:
            # Skip workspace roots themselves
            if crate_dir in workspace_dirs:
                continue

            # Find the closest parent workspace
            is_workspace_member = False
            for workspace_dir in workspace_dirs:
                try:
                    crate_dir.relative_to(workspace_dir)
                    # If we can get a relative path, it's inside this workspace
                    # This is the closest parent due to our sorting
                    if workspace_dir not in workspace_crates:
                        workspace_crates[workspace_dir] = []
                    workspace_crates[workspace_dir].append(crate_dir)
                    is_workspace_member = True
                    break
                except ValueError:
                    # Not relative to this workspace
                    continue

            if not is_workspace_member:
                standalone_crates.append(crate_dir)

        self.workspace_dirs = sorted(workspace_dirs, key=lambda p: str(p))
        self.standalone_crates = standalone_crates
        self.workspace_crates = workspace_crates


def cmd_overview(_args: argparse.Namespace) -> None:
    repo_crates = RepositoryCrates()
    print("Standalone Crates:")
    for crate in repo_crates.standalone_crates:
        print(f"  {crate}")

    print("\nWorkspace Directories:")
    for workspace in repo_crates.workspace_dirs:
        print(f"  {workspace}")

    print("\nWorkspace Crates:")
    for workspace, crates in repo_crates.workspace_crates.items():
        print(f"  Workspace: {workspace}")
        for crate in crates:
            print(f"    {crate}")


def cmd_list_crates(_args: argparse.Namespace) -> None:
    repo_crates = RepositoryCrates()
    all_crates = set(repo_crates.standalone_crates)
    for crates in repo_crates.workspace_crates.values():
        all_crates.update(crates)

    for crate in sorted(all_crates):
        print(crate)


def cmd_list_workspace_crates(args: argparse.Namespace) -> None:
    repo_crates = RepositoryCrates()
    workspace_path = Path(args.workspace)

    # Find the workspace directory that matches the provided path
    workspace_dir = None
    for ws in repo_crates.workspace_dirs:
        if ws == workspace_path or ws.resolve() == workspace_path.resolve():
            workspace_dir = ws
            break

    if workspace_dir is None:
        print(f"Error: Workspace '{args.workspace}' not found")
        print(f"Available workspaces:")
        for ws in repo_crates.workspace_dirs:
            print(f"  {ws}")
        return

    crates = repo_crates.workspace_crates.get(workspace_dir, [])
    for crate in sorted(crates):
        print(crate)


def cmd_list_standalone_crates(_args: argparse.Namespace) -> None:
    repo_crates = RepositoryCrates()
    for crate in sorted(repo_crates.standalone_crates):
        print(crate)


def cmd_list_workspaces(_args: argparse.Namespace) -> None:
    repo_crates = RepositoryCrates()
    for workspace in sorted(repo_crates.workspace_dirs):
        print(workspace)


def main() -> None:
    parser = argparse.ArgumentParser(description='Manage Rust crates in the repository')
    subparsers = parser.add_subparsers(dest='command', help='Available commands')

    # overview subcommand
    parser_overview = subparsers.add_parser('overview', help='Show overview of all crates (default behavior)')
    parser_overview.set_defaults(func=cmd_overview)

    # list-crates subcommand
    parser_list = subparsers.add_parser('list-crates', help='List all crates (workspace and standalone)')
    parser_list.set_defaults(func=cmd_list_crates)

    # list-workspaces subcommand
    parser_workspaces = subparsers.add_parser('list-workspaces', help='List all workspace directories')
    parser_workspaces.set_defaults(func=cmd_list_workspaces)

    # list-workspace-crates subcommand
    parser_workspace = subparsers.add_parser('list-workspace-crates', help='List crates in a specific workspace')
    parser_workspace.add_argument('workspace', help='Path to the workspace directory')
    parser_workspace.set_defaults(func=cmd_list_workspace_crates)

    # list-standalone-crates subcommand
    parser_standalone = subparsers.add_parser('list-standalone-crates', help='List standalone crates')
    parser_standalone.set_defaults(func=cmd_list_standalone_crates)

    args = parser.parse_args()

    # Default to overview if no command is specified
    if args.command is None:
        cmd_overview(args)
    else:
        args.func(args)


if __name__ == "__main__":
    main()
