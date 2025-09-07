import yaml
import os

milestones = [
    {
        "name": "TDD Foundation",
        "card_range": (1, 8),
        "tasks": [
            "Fix config_tests.rs complex type violations",
            "Convert Config struct tests to string functions",
            "Convert ConfigError tests to string outputs",
            "Test `do_*` functions, not internal structs",
            "Fix integration.rs RSB pattern violations",
            "Replace std::env with RSB patterns",
            "Replace std::fs with RSB operations",
            "Replace std::process with shell ops",
            "Establish RED-GREEN cycle discipline",
            "Document TDD workflow",
            "Create TDD templates",
            "Update test runner configuration",
            "Pass all existing tests with TDD patterns"
        ]
    },
    {
        "name": "Core KV Operations",
        "card_range": (6, 12),
        "tasks": [
            "Implement core KV commands (set, get, del, keys, scan)",
            "Develop namespace management",
            "Create basic auth system",
            "Implement stream processing with meta-directives",
            "Define exit code standards",
            "Ensure TDD for each new function",
            "Implement RSB string-first patterns",
            "Add comprehensive error handling",
            "Develop integration test coverage"
        ]
    },
    {
        "name": "TTL & Cache System",
        "card_range": (13, 20),
        "tasks": [
            "Create TTL namespace creation functionality",
            "Implement lazy expiry on read/write operations",
            "Add `--include-expired` flag",
            "Develop cache configuration management",
            "Design eviction policies",
            "Validate near-SQLite performance benchmarks",
            "Verify memory efficiency",
            "Implement concurrent access testing"
        ]
    },
    {
        "name": "Security & Auth",
        "card_range": (21, 28),
        "tasks": [
            "Enforce stream auth preamble",
            "Create user management system",
            "Implement session tokens",
            "Generate and validate API keys",
            "Design configurable security policies",
            "Delegate encryption to system tools",
            "Prevent auth bypass",
            "Implement input validation security",
            "Protect against stream injection"
        ]
    },
    {
        "name": "Data Management",
        "card_range": (29, 35),
        "tasks": [
            "Create backup system with optional encryption",
            "Implement TSV import/export",
            "Validate database integrity",
            "Develop migration utilities",
            "Add data compression options",
            "Verify backup restoration",
            "Test large dataset import/export",
            "Create corruption recovery procedures"
        ]
    },
    {
        "name": "Filesystem Mirror",
        "card_range": (36, 42),
        "tasks": [
            "Implement `export-fs` directory mapping",
            "Create `import-fs` synchronization",
            "Handle context suffix mapping",
            "Manage file formats (JSON default)",
            "Develop incremental sync capabilities",
            "Ensure `grep`/`rg` compatibility",
            "Integrate file system watch",
            "Design conflict resolution policies"
        ]
    }
]

def generate_work_card(milestone, card_number, task):
    card = {
        "card_number": f"CARD_{card_number:03d}",
        "milestone": milestone["name"],
        "feature_name": task,
        "complexity": min(max(len(task) // 5, 1), 10),
        "status": "locked",
        "tests_required": {
            "red_phase": f"Failing test for {task}",
            "green_phase": f"Minimal implementation to pass {task} test",
            "refactor_phase": f"Improve code quality for {task}"
        },
        "dependencies": [],
        "acceptance_criteria": [f"Complete {task} with TDD evidence"],
        "treasure_vault": "SECURED_BY_BEAVER_🦫"
    }
    return card

fortress_path = os.path.dirname(os.path.abspath(__file__))
work_cards_path = os.path.join(fortress_path, "fortress", "work_cards")

for milestone in milestones:
    start, end = milestone["card_range"]
    for idx, task in enumerate(milestone["tasks"], start=start):
        card = generate_work_card(milestone, idx, task)
        card_filename = f"{card['card_number']}_{milestone['name'].lower().replace(' ', '_')}.yml"
        card_filepath = os.path.join(work_cards_path, card_filename)
        
        with open(card_filepath, 'w') as f:
            yaml.safe_dump(card, f, default_flow_style=False)

print("All work cards generated successfully!")