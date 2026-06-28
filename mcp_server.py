from mcp.server.fastmcp import FastMCP
import subprocess
import os

# Initialize the MCP server
mcp = FastMCP("star-toml-cicd")

def run_cicd(args: list[str]) -> str:
    """Helper to run cargo cicd commands in the workspace."""
    cwd = os.environ.get("CARGO_CICD_WORKSPACE", ".")
    try:
        result = subprocess.run(
            ["cargo", "cicd"] + args,
            capture_output=True,
            text=True,
            cwd=cwd
        )
        if result.returncode != 0:
            return f"Command failed with exit code {result.returncode}:\nSTDOUT:\n{result.stdout}\nSTDERR:\n{result.stderr}"
        return result.stdout
    except FileNotFoundError:
        return "Error: `cargo cicd` command not found. Ensure it is built and in PATH."
    except Exception as e:
        return f"Error executing command: {str(e)}"

@mcp.tool()
def doctor_repo() -> str:
    """Diagnose the workspace health. Runs: cargo cicd doctor repo --repo . --json"""
    return run_cicd(["doctor", "repo", "--repo", ".", "--json"])

@mcp.tool()
def gate_repo() -> str:
    """Compute the release gate. Runs: cargo cicd gate repo --repo . --json"""
    return run_cicd(["gate", "repo", "--repo", ".", "--json"])

@mcp.tool()
def verify_repo() -> str:
    """Verify the workspace. Runs: cargo cicd verify repo --repo . --json"""
    return run_cicd(["verify", "repo", "--repo", ".", "--json"])

@mcp.tool()
def trace_profile(profile: str) -> str:
    """
    Execute a trace profile (test, check, clippy, or dry-run). 
    Runs: cargo cicd trace profile --repo . --profile <profile> --json
    """
    if profile not in ["test", "check", "clippy", "dry-run"]:
        return "Error: Invalid profile. Must be one of: test, check, clippy, dry-run."
    return run_cicd(["trace", "profile", "--repo", ".", "--profile", profile, "--json"])

@mcp.tool()
def hooks_pre_tool_use() -> str:
    """Run the pre-tool-use hook barrier check. Runs: cargo cicd hooks pre-tool-use --repo . --json"""
    return run_cicd(["hooks", "pre-tool-use", "--repo", ".", "--json"])

@mcp.tool()
def hooks_install() -> str:
    """Install the antigravity hook provider. Runs: cargo cicd hooks install --repo . --provider antigravity --json"""
    return run_cicd(["hooks", "install", "--repo", ".", "--provider", "antigravity", "--json"])

@mcp.tool()
def hooks_uninstall() -> str:
    """Uninstall the antigravity hook provider. Runs: cargo cicd hooks uninstall --repo . --provider antigravity --json"""
    return run_cicd(["hooks", "uninstall", "--repo", ".", "--provider", "antigravity", "--json"])

@mcp.tool()
def ocel_replay() -> str:
    """Replay and verify the OCEL event log. Runs: cargo cicd ocel replay --repo . --json"""
    return run_cicd(["ocel", "replay", "--repo", ".", "--json"])

@mcp.tool()
def receipt_verify() -> str:
    """Verify all cryptographic receipts. Runs: cargo cicd receipt verify --repo . --json"""
    return run_cicd(["receipt", "verify", "--repo", ".", "--json"])

if __name__ == "__main__":
    mcp.run()
