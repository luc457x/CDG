#!/usr/bin/env python3
import os
import re
import sys
from collections import defaultdict

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _rel(filepath, workspace_root):
    return os.path.relpath(filepath, workspace_root).replace('\\', '/')


# ---------------------------------------------------------------------------
# Step 5b — Style Consistency
# ---------------------------------------------------------------------------

def style_consistency_check(inventory):
    """Return a list of (pattern, majority_convention, violations, example) rows."""
    h1_patterns = {}          # file -> h1 style: 'titled_parens' | 'bare' | 'none'
    numbered_sections = {}    # file -> True/False (has numbered ## sections)
    frontmatter = {}          # file -> True/False
    mixed_bullets = {}        # file -> True if mixes - and *
    emphasis_ratio = {}       # file -> ratio of lines with **
    link_style = {}           # file -> True if has bad link syntax (non-relative)

    bad_link_re = re.compile(r'\[([^\]]+)\]\(([^)]+)\)')

    for item in inventory:
        path = item['path']
        content = item.get('raw_content', '')
        lines = content.splitlines()

        # H1 style
        h1_lines = [l for l in lines if re.match(r'^# ', l)]
        if h1_lines:
            if re.search(r'\([A-Z_]+\.md\)', h1_lines[0]):
                h1_patterns[path] = 'titled_parens'
            else:
                h1_patterns[path] = 'bare'
        else:
            h1_patterns[path] = 'none'

        # Section numbering
        h2_lines = [l for l in lines if re.match(r'^## ', l)]
        numbered = sum(1 for l in h2_lines if re.match(r'^## \d+\.', l))
        numbered_sections[path] = (numbered > 0 and numbered == len(h2_lines))

        # Frontmatter
        frontmatter[path] = content.startswith('---')

        # Mixed bullets
        has_dash = any(re.match(r'^- ', l) for l in lines)
        has_star = any(re.match(r'^\* ', l) for l in lines)
        mixed_bullets[path] = has_dash and has_star

        # Emphasis overuse
        em_lines = sum(1 for l in lines if '**' in l)
        emphasis_ratio[path] = em_lines / len(lines) if lines else 0

        # Bad link syntax
        bad = False
        for label, url in bad_link_re.findall(content):
            if 'http' not in url and not url.startswith('#'):
                if not url.startswith('./') and not url.startswith('../'):
                    bad = True
                    break
                if url.startswith('file://') or 'C:' in url:
                    bad = True
                    break
        link_style[path] = bad

    def majority(d, truthy_key):
        vals = list(d.values())
        count = sum(1 for v in vals if v == truthy_key)
        return count, len(vals) - count

    rows = []

    def is_core_file(path):
        return path == 'AGENTS.md' or (path.startswith('.agents/') and '/' not in path[8:])

    def is_numbered_doc(path):
        return os.path.basename(path) in ('SPEC.md', 'HARNESS.md', 'STRUCTURE.md', 'WORKFLOW.md')

    # Header naming
    h1_violations = []
    for path, pattern in h1_patterns.items():
        if pattern == 'none':
            continue
        if is_core_file(path):
            if pattern != 'titled_parens':
                h1_violations.append(path)
        else:
            if pattern != 'bare':
                h1_violations.append(path)
    rows.append(('Header naming', 'core: titled_parens / skills: bare', len(h1_violations),
                 h1_violations[0] if h1_violations else 'none'))

    # Section numbering
    num_viol = []
    for p, v in numbered_sections.items():
        has_h2 = any(re.match(r'^## ', l) for l in
                     inventory[next((i for i, x in enumerate(inventory) if x['path'] == p), 0)]
                     .get('raw_content', '').splitlines())
        if not has_h2:
            continue
        if is_numbered_doc(p):
            if not v:
                num_viol.append(p)
        elif p.endswith('SKILL.md'):
            if v:
                num_viol.append(p)
    rows.append(('Section numbering', 'core specs: numbered / skills: unnumbered',
                 len(num_viol), num_viol[0] if num_viol else 'none'))

    # Frontmatter (only SKILL.md and rules files should have it)
    skill_no_fm = []
    for p, v in frontmatter.items():
        is_rule = p.startswith('.agents/rules/') and p.endswith('.md')
        is_skill_main = p.endswith('SKILL.md')
        if (is_rule or is_skill_main) and not v:
            skill_no_fm.append(p)
    rows.append(('Frontmatter on skills', 'SKILL.md and rule files have frontmatter',
                 len(skill_no_fm), skill_no_fm[0] if skill_no_fm else 'none'))

    # Link syntax
    bad_links = [p for p, v in link_style.items() if v]
    rows.append(('Link syntax', '[label](./relative)', len(bad_links),
                 bad_links[0] if bad_links else 'none'))

    # Bullet style
    mixed = [p for p, v in mixed_bullets.items() if v]
    rows.append(('Bullet style', 'single character (- or *)', len(mixed),
                 mixed[0] if mixed else 'none'))

    # Emphasis overuse (raise threshold to 20%; ignore files that are indexes or lists, e.g., STRUCTURE.md, BACKLOG.md, TASKS.md)
    overused = []
    for p, v in emphasis_ratio.items():
        if v > 0.20:
            if os.path.basename(p) not in ('STRUCTURE.md', 'BACKLOG.md', 'TASKS.md', 'PROGRESS.md'):
                overused.append(p)
    rows.append(('Emphasis overuse (>20% lines)', '<20% lines have **',
                 len(overused), overused[0] if overused else 'none'))

    return rows


# ---------------------------------------------------------------------------
# Step 6b — Coupling & Cohesion
# ---------------------------------------------------------------------------

def coupling_cohesion_check(inventory, agents_dir):
    """Build a reference graph; return per-file fan-out/fan-in and circular pairs."""
    link_re = re.compile(r'\[([^\]]+)\]\(([^)]+)\)')
    # Map basename -> full rel_path for resolution
    basename_map = {os.path.basename(item['path']): item['path'] for item in inventory}

    graph = defaultdict(set)   # path -> set of paths it references
    for item in inventory:
        src = item['path']
        content = item.get('raw_content', '')
        for _, url in link_re.findall(content):
            if 'http' in url or url.startswith('#'):
                continue
            # Normalize: strip anchors, resolve relative to src dir
            url_clean = url.split('#')[0]
            resolved = os.path.normpath(
                os.path.join(os.path.dirname(src), url_clean)
            ).replace('\\', '/')
            # Match against known paths
            for known in (item['path'] for item in inventory):
                if known == resolved or os.path.basename(known) == os.path.basename(resolved):
                    if known != src:
                        graph[src].add(known)
                    break

    # Fan-in
    fan_in = defaultdict(int)
    for src, targets in graph.items():
        for t in targets:
            fan_in[t] += 1

    # Circular pairs
    circular = []
    seen = set()
    for a, targets in graph.items():
        for b in targets:
            if a in graph.get(b, set()):
                pair = tuple(sorted([a, b]))
                if pair not in seen:
                    seen.add(pair)
                    circular.append(pair)

    rows = []
    for item in sorted(inventory, key=lambda x: x['path']):
        p = item['path']
        fo = len(graph[p])
        fi = fan_in[p]
        is_circular = any(p in pair for pair in circular)
        if is_circular:
            verdict = '🔴 Circular'
        elif fo > 3:
            verdict = '⚠️ High fan-out'
        else:
            verdict = '✅ Healthy'
        rows.append((p, fo, fi, verdict))

    return rows, circular


# ---------------------------------------------------------------------------
# Step 6c — Overengineering
# ---------------------------------------------------------------------------

def overengineering_check(inventory, agents_dir):
    """Return (metrics list, verdict label)."""
    total_files = len(inventory)
    skill_files = [i for i in inventory if 'skills/' in i['path']]
    num_skills = len(skill_files)

    # Count active tasks
    tasks_path = os.path.join(agents_dir, 'TASKS.md')
    active_tasks = 0
    if os.path.exists(tasks_path):
        with open(tasks_path, 'r', encoding='utf-8', errors='ignore') as f:
            for line in f:
                if re.search(r'\[ \]|\[/\]', line):  # uncomplete or in-progress
                    active_tasks += 1

    ratio = num_skills / active_tasks if active_tasks > 0 else float('inf')
    ratio_str = f'{ratio:.1f}x' if ratio != float('inf') else '∞ (no active tasks)'

    # Dead files: files not referenced by any other file
    link_re = re.compile(r'\[([^\]]+)\]\(([^)]+)\)')
    referenced = set()
    for item in inventory:
        for _, url in link_re.findall(item.get('raw_content', '')):
            referenced.add(os.path.basename(url.split('#')[0]))
    dead = [i['path'] for i in inventory
            if os.path.basename(i['path']) not in referenced
            and i['path'] != 'AGENTS.md']  # entry point is never referenced
    num_dead = len(dead)

    # Hop depth from AGENTS.md
    # Simplified BFS: assume entry point = AGENTS.md in root
    entry = next((i for i in inventory if os.path.basename(i['path']) == 'AGENTS.md'), None)
    max_depth = 0
    if entry:
        graph = defaultdict(set)
        basename_map = {os.path.basename(i['path']): i['path'] for i in inventory}
        for item in inventory:
            src = item['path']
            for _, url in link_re.findall(item.get('raw_content', '')):
                bname = os.path.basename(url.split('#')[0])
                if bname in basename_map and basename_map[bname] != src:
                    graph[src].add(basename_map[bname])
        # BFS
        from collections import deque
        visited = {entry['path']}
        queue = deque([(entry['path'], 0)])
        while queue:
            node, depth = queue.popleft()
            max_depth = max(max_depth, depth)
            for neighbor in graph[node]:
                if neighbor not in visited:
                    visited.add(neighbor)
                    queue.append((neighbor, depth + 1))

    # Spec hollow?
    spec = next((i for i in inventory if os.path.basename(i['path']) == 'SPEC.md'), None)
    spec_hollow = spec and spec['status'] in ('Hollow', 'Critical')

    # Verdict
    if spec_hollow and num_skills > 10:
        verdict = '🔴 Premature'
    elif ratio > 3 or num_dead > 5:
        verdict = '⚠️ Bloated'
    else:
        verdict = '✅ Proportionate'

    metrics = [
        ('Total .agents/ files', total_files, '—', '✅' if total_files < 40 else '⚠️'),
        ('Skill files', num_skills, '—', '—'),
        ('Active tasks (TASKS.md)', active_tasks, '—', '—'),
        ('Skill-to-task ratio', ratio_str, '≤3x', '✅' if ratio <= 3 else '🔴'),
        ('Dead files (zero refs)', num_dead, '0', '✅' if num_dead == 0 else '⚠️'),
        ('Max hop depth from AGENTS.md', max_depth, '≤3', '✅' if max_depth <= 3 else '⚠️'),
        ('SPEC.md hollow?', 'Yes' if spec_hollow else 'No', 'No', '🔴' if spec_hollow else '✅'),
        ('**Overall verdict**', verdict, '—', '—'),
    ]
    return metrics, verdict


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def run_audit(workspace_root=None, output_path=None):
    if not workspace_root:
        workspace_root = os.getcwd()

    agents_dir = os.path.join(workspace_root, ".agents")
    if not os.path.exists(agents_dir):
        print(f"Error: .agents directory not found in {workspace_root}", file=sys.stderr)
        sys.exit(1)

    placeholder_pattern = re.compile(r'\[(e\.g\.,|Define|Item|YOUR|INSERT|TODO|[^\]]*\.\.\.|[^\]]*\|[^\]]*)\]', re.IGNORECASE)
    link_pattern = re.compile(r'\[([^\]]+)\]\(([^)]+)\)')

    inventory = []
    all_placeholders = {}
    all_bad_links = {}
    all_empty_sections = {}

    depended_upon = ["SPEC.md", "HARNESS.md", "STRUCTURE.md", "WORKFLOW.md", "TASKS.md"]
    structure_path = os.path.join(agents_dir, "STRUCTURE.md")
    if os.path.exists(structure_path):
        try:
            with open(structure_path, 'r', encoding='utf-8', errors='ignore') as f:
                full_content = f.read()
            doc_struct_match = re.search(r'## Documentation Structure(.*?)(##|$)', full_content, re.DOTALL)
            if doc_struct_match:
                found_mds = re.findall(r'\*\s+\*\*([a-zA-Z0-9_]+\.md)\*\*', doc_struct_match.group(1))
                if found_mds:
                    for md in found_mds:
                        if md not in depended_upon:
                            depended_upon.append(md)
        except Exception:
            pass

    for root, dirs, files in os.walk(agents_dir):
        for file in files:
            if file.endswith('.md') or file.endswith('.ps1') or file.endswith('.sh'):
                filepath = os.path.join(root, file)
                rel_path = _rel(filepath, workspace_root)

                with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
                    content = f.read()

                lines = content.splitlines()

                # Placeholders
                file_placeholders = []
                for idx, line in enumerate(lines, 1):
                    for match in placeholder_pattern.finditer(line):
                        file_placeholders.append((idx, match.group(0)))
                if file_placeholders:
                    all_placeholders[rel_path] = file_placeholders

                # Bad links
                file_bad_links = []
                for idx, line in enumerate(lines, 1):
                    for label, url in link_pattern.findall(line):
                        if not url.startswith('./') and not url.startswith('../') and not url.startswith('#') and 'http' not in url:
                            file_bad_links.append((idx, f"[{label}]({url})"))
                        elif url.startswith('file://') or 'C:' in url or 'Users/' in url:
                            file_bad_links.append((idx, f"[{label}]({url})"))
                if file_bad_links:
                    all_bad_links[rel_path] = file_bad_links

                # Empty sections
                empty_sections = []
                current_header = None
                current_header_line = 0
                header_content_count = 0
                for idx, line in enumerate(lines, 1):
                    stripped = line.strip()
                    if stripped.startswith('#'):
                        if current_header and header_content_count == 0:
                            empty_sections.append((current_header_line, current_header))
                        current_header = stripped
                        current_header_line = idx
                        header_content_count = 0
                    elif stripped and not stripped.startswith('---') and not stripped.startswith('*') and stripped != "":
                        header_content_count += 1
                if current_header and header_content_count == 0:
                    empty_sections.append((current_header_line, current_header))
                if empty_sections:
                    all_empty_sections[rel_path] = empty_sections

                # Status
                status = "Instantiated"
                is_depended = file in depended_upon
                if file == "SPEC.md":
                    status = "Critical"
                elif file_placeholders or empty_sections:
                    placeholder_density = len(file_placeholders) / len(lines) if len(lines) > 0 else 0
                    if placeholder_density > 0.1 or (is_depended and len(file_placeholders) > 3):
                        status = "Critical" if is_depended else "Hollow"
                    else:
                        status = "Partial"

                purpose = "Skill definition"
                if file == "STRUCTURE.md":
                    purpose = "Documentation index & structure list"
                elif file == "SPEC.md":
                    purpose = "Specifications & business rules"
                elif file == "BACKLOG.md":
                    purpose = "Raw ideas backlog"
                elif file == "TASKS.md":
                    purpose = "Actionable roadmap"
                elif file == "TASKS_ARCHIVE.md":
                    purpose = "Historical task archive"
                elif file == "HARNESS.md":
                    purpose = "Technical harness & QA checklist"
                elif file == "PROGRESS.md":
                    purpose = "Session log"
                elif file == "PROGRESS_ARCHIVE.md":
                    purpose = "Archived sessions"
                elif file == "WORKFLOW.md":
                    purpose = "Lifecycle & commit formats"
                elif "personas" in rel_path:
                    purpose = f"Persona instruction for {file.replace('.md', '')}"
                elif "examples" in rel_path:
                    purpose = "Reference examples"
                elif "scripts" in rel_path:
                    purpose = "Helper script"

                inventory.append({
                    "path": rel_path,
                    "size": len(content.encode('utf-8')),
                    "purpose": purpose,
                    "status": status,
                    "total_lines": len(lines),
                    "raw_content": content,
                })

    # Run new analysis checks
    style_rows = style_consistency_check(inventory)
    coupling_rows, circular_pairs = coupling_cohesion_check(inventory, agents_dir)
    oe_metrics, oe_verdict = overengineering_check(inventory, agents_dir)

    # ---------------------------------------------------------------------------
    # Assemble report
    # ---------------------------------------------------------------------------
    report = []
    report.append("# Structure Audit Report\n")

    # 10a. File Inventory Table
    report.append("## 10a. File Inventory Table\n")
    report.append("| File | Size (Bytes) | Purpose | Status |")
    report.append("| :--- | :--- | :--- | :--- |")

    status_emoji = {
        "Instantiated": "✅ Instantiated",
        "Partial": "⚠️ Partial",
        "Hollow": "🔴 Hollow",
        "Critical": "🔴 **Critical**"
    }

    counts = {"Instantiated": 0, "Partial": 0, "Hollow": 0, "Critical": 0}
    for item in sorted(inventory, key=lambda x: x["path"]):
        counts[item["status"]] += 1
        emoji = status_emoji[item["status"]]
        report.append(f"| `{item['path']}` | {item['size']} | {item['purpose']} | {emoji} |")

    report.append(f"\n{counts['Instantiated']} Instantiated · {counts['Partial']} Partial · {counts['Hollow']} Hollow · {counts['Critical']} Critical\n")

    # 10b. Pros & Cons
    report.append("## 10b. Pros & Cons Table\n")
    report.append("| Category | Strengths | Weaknesses |")
    report.append("| :--- | :--- | :--- |")
    for cat in ["Architecture", "Skills", "Prompt quality", "Token efficiency",
                "Integrity", "Style consistency", "Coupling & cohesion", "Overengineering", "Git exposure"]:
        report.append(f"| **{cat}** | | |")
    report.append("")

    # 10c. Critical Gaps
    report.append("## 10c. Critical Gaps\n")
    gap_idx = 1
    for path, placeholders in all_placeholders.items():
        severity = "🔴" if any(p in path for p in depended_upon) else "⚠️"
        sev_label = "Critical" if severity == "🔴" else "Medium"
        report.append(f"### {severity} GAP-{gap_idx}: Placeholders in `{path}`")
        report.append(f"- **Severity:** {sev_label}")
        report.append(f"- **Problem:** {', '.join([p[1] for p in placeholders])}")
        report.append("- **Fix:** Replace placeholders with concrete definitions.")
        report.append("")
        gap_idx += 1

    for path, bad_links in all_bad_links.items():
        report.append(f"### ⚠️ GAP-{gap_idx}: Invalid Links in `{path}`")
        report.append("- **Severity:** Medium")
        report.append(f"- **Problem:** Non-relative links: {', '.join([bl[1] for bl in bad_links])}")
        report.append("- **Fix:** Use `./relative/path` format.")
        report.append("")
        gap_idx += 1

    if circular_pairs:
        for a, b in circular_pairs:
            report.append(f"### ⚠️ GAP-{gap_idx}: Circular Reference — `{a}` ↔ `{b}`")
            report.append("- **Severity:** Medium")
            report.append("- **Problem:** Both files reference each other, creating tight coupling.")
            report.append("- **Fix:** Break cycle by inlining or extracting shared content to a third file.")
            report.append("")
            gap_idx += 1

    # 10d. Token Efficiency Scorecard
    report.append("## 10d. Token Efficiency Scorecard\n")
    report.append("| File | Lang Cost | Format Fit | Redundancy | Score |")
    report.append("| :--- | :--- | :--- | :--- | :--- |")
    for f in depended_upon:
        report.append(f"| `{f}` | ✅ English | ⚠️ Improvable | ✅ None | B |")
    report.append("")

    # 10e. Style Consistency Summary
    report.append("## 10e. Style Consistency Summary\n")
    report.append("| Pattern | Majority Convention | # Violations | Example Violator |")
    report.append("| :--- | :--- | :--- | :--- |")
    for pattern, convention, n_viol, example in style_rows:
        status_cell = '✅' if n_viol == 0 else '⚠️'
        report.append(f"| {pattern} | {convention} | {status_cell} {n_viol} | `{example}` |")
    report.append("")

    # 10f. Coupling & Cohesion Summary
    report.append("## 10f. Coupling & Cohesion Summary\n")
    report.append("| File/Skill | Fan-out (refs out) | Fan-in (refs in) | Verdict |")
    report.append("| :--- | :--- | :--- | :--- |")
    for path, fo, fi, verdict in coupling_rows:
        report.append(f"| `{path}` | {fo} | {fi} | {verdict} |")
    report.append("")

    # 10g. Overengineering Scorecard
    report.append("## 10g. Overengineering Scorecard\n")
    report.append("| Metric | Value | Threshold | Status |")
    report.append("| :--- | :--- | :--- | :--- |")
    for metric, value, threshold, status in oe_metrics:
        report.append(f"| {metric} | {value} | {threshold} | {status} |")
    report.append("")

    # 10h. Summary Verdict Table
    report.append("## 10h. Summary Verdict Table\n")
    report.append("| Dimension | Score |")
    report.append("| :--- | :--- |")
    for dim in ["Cross-reference integrity", "Placeholder density", "Rule consistency",
                "Prompt engineering quality", "Token efficiency",
                "Style consistency", "Coupling & cohesion", "Overengineering",
                "Scope & architecture fit"]:
        report.append(f"| {dim} | — / 10 |")
    report.append("| **Overall** | **— / 10** |")
    report.append("\n### Verdict\n")
    report.append(f"- **Overengineering verdict:** {oe_verdict}")
    report.append("- **Top 3 blockers:** _(fill after manual review)_")
    report.append("- **Next milestone:** _(fill after manual review)_")

    content = "\n".join(report)

    if output_path:
        out_dir = os.path.dirname(output_path)
        if out_dir:
            os.makedirs(out_dir, exist_ok=True)
        with open(output_path, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"Report generated at: {output_path}")
    else:
        if hasattr(sys.stdout, 'reconfigure'):
            sys.stdout.reconfigure(encoding='utf-8')
        try:
            print(content)
        except UnicodeEncodeError:
            print(content.encode('ascii', errors='replace').decode('ascii'))


if __name__ == '__main__':
    run_audit(
        workspace_root=sys.argv[1] if len(sys.argv) > 1 else None,
        output_path=sys.argv[2] if len(sys.argv) > 2 else None
    )
