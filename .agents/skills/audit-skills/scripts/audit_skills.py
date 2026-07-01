import os
import re
import sys
import argparse

sys.stdout.reconfigure(encoding='utf-8')

def parse_simple_yaml(yaml_text):
    data = {}
    current_key = None
    for line in yaml_text.splitlines():
        if not line.strip():
            continue
        m = re.match(r"^([a-zA-Z0-9_-]+)\s*:\s*(.*)$", line)
        if m:
            current_key = m.group(1).strip()
            val = m.group(2).strip()
            if val.startswith(">"):
                data[current_key] = ""
            elif (val.startswith('"') and val.endswith('"')) or (val.startswith("'") and val.endswith("'")):
                data[current_key] = val[1:-1]
            else:
                data[current_key] = val
        elif current_key and line.startswith(" "):
            data[current_key] = (data[current_key] + " " + line.strip()).strip()
    return data

def check_links(file_path, content):
    # Strip fenced code blocks
    content_clean = re.sub(r"```.*?```", "", content, flags=re.DOTALL)
    # Strip inline code backticks
    content_clean = re.sub(r"`.*?`", "", content_clean)

    broken_links = []
    # Find all markdown links [text](link)
    # Exclude external links starting with http, https, mailto, or page hashes
    links = re.findall(r"\[([^\]]+)\]\(([^)]+)\)", content_clean)
    file_dir = os.path.dirname(file_path)
    
    for text, link in links:
        if link.startswith(("http://", "https://", "mailto:", "#")):
            continue
            
        clean_link = link.split("#")[0]
        if not clean_link:
            continue
            
        target_path = os.path.normpath(os.path.join(file_dir, clean_link))
        if not os.path.exists(target_path):
            broken_links.append((link, target_path))
            
    return broken_links

def analyze_skill_file(skill_path, skills_dir, max_lines=180):
    with open(skill_path, "r", encoding="utf-8") as f:
        content = f.read()

    lines = content.splitlines()
    line_count = len(lines)

    has_frontmatter = False
    frontmatter = {}
    if content.startswith("---"):
        parts = content.split("---", 2)
        if len(parts) >= 3:
            has_frontmatter = True
            frontmatter_raw = parts[1]
            try:
                frontmatter = parse_simple_yaml(frontmatter_raw)
            except Exception as e:
                frontmatter = {"error": str(e)}

    # Get skill name from directory name
    rel_path = os.path.relpath(skill_path, skills_dir)
    path_parts = rel_path.split(os.sep)
    skill_name = path_parts[1] if len(path_parts) > 2 else path_parts[0]

    desc = frontmatter.get("description", "")
    desc_len = len(desc)
    when_to_use = frontmatter.get("when_to_use", "")
    when_to_use_len = len(when_to_use)

    sections = []
    for line in lines:
        m = re.match(r"^(#{1,3})\s+(.*)$", line)
        if m:
            sections.append((len(m.group(1)), m.group(2).strip()))

    # Clean code blocks and inline code to calculate prose article ratio
    content_clean = re.sub(r"```.*?```", "", content, flags=re.DOTALL)
    content_clean = re.sub(r"`.*?`", "", content_clean)
    words = re.findall(r"\b\w+\b", content_clean.lower())
    articles = [w for w in words if w in ("the", "a", "an")]
    article_ratio = len(articles) / max(1, len(words))

    broken_links = check_links(skill_path, content)

    return {
        "path": rel_path,
        "name": skill_name,
        "line_count": line_count,
        "has_frontmatter": has_frontmatter,
        "frontmatter_error": frontmatter.get("error", None),
        "frontmatter_name": frontmatter.get("name", None),
        "description": desc,
        "desc_len": desc_len,
        "when_to_use": when_to_use,
        "when_to_use_len": when_to_use_len,
        "sections": sections,
        "article_ratio": article_ratio,
        "broken_links": broken_links
    }

def main():
    parser = argparse.ArgumentParser(description="Audit agent skills for quality and compliance.")
    parser.add_argument("--dir", "-d", help="Path to skills directory (defaults to current dir/.agents/skills or relative script path)")
    parser.add_argument("--skill", "-s", help="Audit only this specific skill (matches by directory name)")
    parser.add_argument("--max-lines", type=int, default=180, help="Maximum allowed line count for SKILL.md (default: 180)")
    parser.add_argument("--output", "-o", help="Path to output markdown report file")
    args = parser.parse_args()

    skills_dir = args.dir
    if not skills_dir:
        # Try finding in current directory
        cwd_skills = os.path.join(os.getcwd(), ".agents", "skills")
        if os.path.exists(cwd_skills):
            skills_dir = cwd_skills
        else:
            # Fallback relative to script location (assuming scripts/audit_skills.py)
            script_dir = os.path.dirname(os.path.abspath(__file__))
            fallback_dir = os.path.normpath(os.path.join(script_dir, "..", "..", ".."))
            if os.path.exists(fallback_dir) and os.path.basename(fallback_dir) == "skills":
                skills_dir = fallback_dir
            else:
                skills_dir = os.getcwd()

    if not os.path.exists(skills_dir):
        print(f"Error: Skills directory '{skills_dir}' does not exist.")
        sys.exit(1)

    if args.skill:
        print(f"Auditing skill '{args.skill}' under: {os.path.abspath(skills_dir)}")
    else:
        print(f"Auditing all skills under: {os.path.abspath(skills_dir)}")

    all_md_files = []
    for root, dirs, files in os.walk(skills_dir):
        for f in files:
            if f.endswith(".md"):
                all_md_files.append(os.path.join(root, f))

    results = []
    for skill_path in all_md_files:
        is_main_skill = os.path.basename(skill_path) == "SKILL.md"
        try:
            res = analyze_skill_file(skill_path, skills_dir, max_lines=args.max_lines)
            if args.skill and res["name"].lower() != args.skill.lower():
                continue
            results.append((is_main_skill, res))
        except Exception as e:
            print(f"Error reading {skill_path}: {e}")

    # Output report
    report = []
    report.append("# Skills Quality Audit Report\n")
    
    # 1. Broken Links Check
    broken_count = 0
    broken_section_output = []
    for is_main, r in sorted(results, key=lambda x: (x[1]["name"], not x[0], x[1]["path"])):
        if r["broken_links"]:
            broken_section_output.append(f"### Broken links in `{r['path']}`:")
            for link, abs_t in r["broken_links"]:
                broken_section_output.append(f"- `{link}` (Resolved: `{abs_t}`)")
                broken_count += 1
            broken_section_output.append("")
            
    report.append("## 1. Broken Links")
    if broken_count > 0:
        report.append(f"Found {broken_count} broken relative links:\n")
        report.append("\n".join(broken_section_output))
    else:
        report.append("✅ No broken relative links found.\n")

    # 2. Main SKILL.md Structure & Rules
    report.append("## 2. SKILL.md Consistency")
    skill_issues = []
    for is_main, r in sorted(results, key=lambda x: (x[1]["name"], x[1]["path"])):
        if not is_main:
            continue
            
        issues = []
        if r["line_count"] > args.max_lines:
            issues.append(f"Line count ({r['line_count']}) exceeds threshold ({args.max_lines})")
        if not r["has_frontmatter"]:
            issues.append("Missing YAML frontmatter header")
        elif r["frontmatter_error"]:
            issues.append(f"YAML parsing error: {r['frontmatter_error']}")
        elif r["frontmatter_name"] != r["name"]:
            issues.append(f"YAML name '{r['frontmatter_name']}' does not match directory '{r['name']}'")
            
        if r["has_frontmatter"] and not r["when_to_use"]:
            issues.append("Missing 'when_to_use' trigger field in YAML frontmatter")
        if r["desc_len"] > 300:
            issues.append(f"Description length ({r['desc_len']}) exceeds 300 characters. Keep frontmatter description concise.")
        if r["when_to_use_len"] > 300:
            issues.append(f"when_to_use length ({r['when_to_use_len']}) exceeds 300 characters. Keep triggers concise.")

        has_workflows = any(any(w in s[1].lower() for w in ["workflow", "process", "step", "rules"]) for s in r["sections"])
        if not has_workflows:
            issues.append("Missing Workflows, Process, Rules, or Step section")

        # Enforce caveman style: no articles in YAML description / when_to_use, and prose ratio <= 2.0%
        if r["has_frontmatter"]:
            desc_words = re.findall(r"\b\w+\b", r["description"].lower())
            desc_articles = [w for w in desc_words if w in ("the", "a", "an")]
            if desc_articles:
                issues.append(f"Description contains articles {set(desc_articles)}. Enforce caveman style (drop a/an/the in frontmatter description).")
            
            wtu_words = re.findall(r"\b\w+\b", r["when_to_use"].lower())
            wtu_articles = [w for w in wtu_words if w in ("the", "a", "an")]
            if wtu_articles:
                issues.append(f"when_to_use contains articles {set(wtu_articles)}. Enforce caveman style (drop a/an/the in frontmatter when_to_use).")
        
        if r["article_ratio"] > 0.02:
            issues.append(f"Prose article ratio ({r['article_ratio']:.2%}) exceeds 2.0% threshold. Remove 'a', 'an', 'the' to enforce caveman style.")

        if issues:
            skill_issues.append((r["name"], r["path"], issues))

    if skill_issues:
        report.append("Found structural issues in the following skills:\n")
        for name, path, issues in skill_issues:
            report.append(f"### Skill: `{name}`")
            report.append(f"- File: `{path}`")
            for iss in issues:
                report.append(f"- ⚠️ {iss}")
            report.append("")
    else:
        report.append("✅ All skills follow structural guidelines perfectly.\n")

    # Summary
    total_skills = len([r for is_main, r in results if is_main])
    report.append("## Summary")
    report.append(f"- Total skills checked: {total_skills}")
    report.append(f"- Total markdown files scanned: {len(results)}")
    report.append(f"- Total broken links: {broken_count}")
    report.append(f"- Total skills with structural issues: {len(skill_issues)}")

    report_content = "\n".join(report)

    if args.output:
        out_dir = os.path.dirname(args.output)
        if out_dir:
            os.makedirs(out_dir, exist_ok=True)
        with open(args.output, 'w', encoding='utf-8') as f:
            f.write(report_content)
        print(f"Report generated at: {args.output}")
    else:
        print(report_content)

    if broken_count > 0 or len(skill_issues) > 0:
        sys.exit(1)
    else:
        sys.exit(0)

if __name__ == "__main__":
    main()
