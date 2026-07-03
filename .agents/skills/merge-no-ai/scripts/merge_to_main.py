import subprocess
import sys
import os

def run_cmd(args):
    result = subprocess.run(args, capture_output=True, text=True)
    if result.returncode != 0:
        raise Exception(f"Command {' '.join(args)} failed:\nSTDOUT:\n{result.stdout}\nSTDERR:\n{result.stderr}")
    return result.stdout.strip()

def main():
    # Store starting branch
    start_branch = None
    try:
        # 1. Check if git repo
        if not os.path.exists(".git"):
            print("Error: Not in a git repository.")
            sys.exit(1)

        # Get current branch
        start_branch = run_cmd(["git", "branch", "--show-current"])
        if not start_branch:
            print("Error: Not on any branch (detached HEAD?).")
            sys.exit(1)

        if start_branch == "main":
            print("Error: You are already on main branch. Checkout your development branch (e.g. dev) first.")
            sys.exit(1)

        # Check clean workspace
        status = run_cmd(["git", "status", "--porcelain"])
        if status:
            print("Error: Git working tree is not clean. Commit or stash changes first.")
            sys.exit(1)

        print(f"Starting merge workflow from branch '{start_branch}'...")

        # 2. Switch to main
        print("Checking out main branch...")
        run_cmd(["git", "checkout", "main"])

        # 3. Check and update .gitignore on main
        gitignore_path = ".gitignore"
        ai_patterns = [".agents/", "AGENTS.md", ".agentignore"]
        
        if os.path.exists(gitignore_path):
            with open(gitignore_path, "r", encoding="utf-8") as f:
                lines = f.read().splitlines()
            
            # Identify which AI documentation patterns are uncommented
            uncommented = []
            for pattern in ai_patterns:
                for line in lines:
                    line_strip = line.strip()
                    if line_strip == pattern or line_strip.startswith(pattern + "/"):
                        uncommented.append(pattern)
                        break
            
            missing_patterns = [p for p in ai_patterns if p not in uncommented]
            if missing_patterns:
                print(f"Uncommenting missing AI documentation patterns in .gitignore on main: {missing_patterns}")
                new_lines = []
                for line in lines:
                    line_strip = line.strip()
                    fixed = False
                    for pattern in missing_patterns:
                        if line_strip == f"# {pattern}":
                            new_lines.append(pattern)
                            fixed = True
                            break
                    if not fixed:
                        new_lines.append(line)
                
                with open(gitignore_path, "w", encoding="utf-8") as f:
                    f.write("\n".join(new_lines) + "\n")
                
                run_cmd(["git", "add", ".gitignore"])
                run_cmd(["git", "commit", "-m", "chore: ignore AI documentation on main branch"])
                print(".gitignore updated and committed on main.")
        else:
            print("Warning: .gitignore file not found on main.")

        # 4. Check if any AI documentation is currently tracked on main
        tracked_files = []
        for pattern in ai_patterns:
            files_out = run_cmd(["git", "ls-files", pattern])
            if files_out:
                tracked_files.extend(files_out.splitlines())
        
        if tracked_files:
            print(f"AI documentation files currently tracked on main: {tracked_files}")
            print("Removing tracking for these files on main branch...")
            run_cmd(["git", "rm", "-r", "--cached"] + tracked_files)
            run_cmd(["git", "commit", "-m", "chore: untrack AI documentation on main"])
            print("Untracking committed on main.")

        # 5. Merge start_branch into main without committing
        print(f"Merging {start_branch} into main (no-commit)...")
        merge_out = run_cmd(["git", "merge", start_branch, "--no-commit", "--no-ff"])
        
        if "Already up to date." in merge_out:
            print("Already up to date. No merge commit needed.")
        else:
            # 6. Untrack AI documentation from the merge stage
            print("Ensuring AI documentation is not staged in merge commit...")
            run_cmd(["git", "rm", "-r", "--cached", ".agents", "AGENTS.md", ".agentignore", "--ignore-unmatch"])
            
            # 7. Commit the merge
            run_cmd(["git", "commit", "-m", f"Merge branch '{start_branch}' into main (excluding AI documentation)"])
            print("Merge committed successfully.")

    except Exception as e:
        print(f"Error during merge workflow: {e}")
        # Try to abort merge if in progress
        try:
            run_cmd(["git", "merge", "--abort"])
        except Exception:
            pass
        sys.exit(1)

    finally:
        # Switch back to start_branch
        if start_branch:
            try:
                run_cmd(["git", "checkout", "-f", start_branch])
                print(f"Switched back to development branch '{start_branch}'.")
            except Exception as restore_err:
                print(f"Fatal: Could not checkout branch '{start_branch}': {restore_err}")

if __name__ == "__main__":
    main()
