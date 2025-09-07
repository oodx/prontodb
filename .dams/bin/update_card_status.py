#!/usr/bin/env python3
"""
Teddy's Card Status Management Script
Updates work card status through RED-GREEN-REFACTOR cycle
"""

import sys
import yaml
from pathlib import Path

# Status transitions
VALID_STATUSES = {
    "🟡 CURRENT": "Current card in progress",
    "🔴 RED": "Test fails as expected", 
    "🟢 GREEN": "Test passes with minimal implementation",
    "🔵 REFACTOR": "Test passes with clean refactored code",
    "✅ APPROVED": "Card completed and approved"
}

VALID_TRANSITIONS = {
    "🟡 CURRENT": ["🔴 RED"],
    "🔴 RED": ["🟢 GREEN", "🟡 CURRENT"],  # Can regress to CURRENT if needed
    "🟢 GREEN": ["🔵 REFACTOR", "🔴 RED"],  # Can regress to RED if issues found
    "🔵 REFACTOR": ["✅ APPROVED", "🟢 GREEN"],  # Can regress to GREEN if refactor fails
    "✅ APPROVED": []  # Final state
}

def load_card(card_path):
    """Load card YAML file"""
    try:
        with open(card_path, 'r') as f:
            return yaml.safe_load(f)
    except Exception as e:
        print(f"❌ Error loading card: {e}")
        sys.exit(1)

def save_card(card_path, card_data):
    """Save card YAML file"""
    try:
        with open(card_path, 'w') as f:
            yaml.dump(card_data, f, default_flow_style=False, sort_keys=False)
        print(f"💾 Card saved: {card_path}")
    except Exception as e:
        print(f"❌ Error saving card: {e}")
        sys.exit(1)

def find_current_card():
    """Find which card is currently active (status: 🟡 CURRENT)"""
    current_cards = []
    
    # Check all CARD_*.yml files in project root
    for card_file in Path(".").glob("CARD_*.yml"):
        try:
            card_data = load_card(card_file)
            if card_data.get('status') == '🟡 CURRENT':
                card_num = int(card_file.stem.split('_')[1])
                current_cards.append((card_num, card_file))
        except:
            continue
    
    return current_cards

def check_project_root_discipline():
    """BEAVER PROJECT MANAGEMENT: Enforce single card rule in project root"""
    root_cards = list(Path(".").glob("CARD_*.yml"))
    
    if len(root_cards) > 1:
        print(f"🦫 BEAVER PROJECT MANAGEMENT RAGE: {len(root_cards)} cards in project root!")
        print(f"🦫 TYRANNICAL PROJECT RULE: Only ONE card allowed in root at a time!")
        print(f"   Published cards found:")
        for card in sorted(root_cards):
            card_data = load_card(card)
            status = card_data.get('status', 'UNKNOWN')
            print(f"   - {card.name}: {status}")
        print(f"🦫 BEAVER DECREE: Archive completed cards to vault, keep only current!")
        print(f"   Example: mv CARD_001.yml .dams/archive/")
        sys.exit(1)
    
    return root_cards

def update_status(card_number, new_status):
    """Update card status with validation"""
    
    # BEAVER PROJECT MANAGEMENT: Check root card discipline FIRST
    root_cards = check_project_root_discipline()
    
    # Find card file - MUST be in project root (beaver paranoia!)
    card_path = Path(f"CARD_{card_number:03d}.yml")
    if not card_path.exists():
        # Check if it's hiding in the dam vault
        vault_path = Path(f".dams/CARD_{card_number:03d}.yml")
        if vault_path.exists():
            print(f"🦫 BEAVER RAGE: Card {card_number:03d} is SECURED in the dam vault!")
            print(f"   Cards must be published to PROJECT ROOT for status updates!")
            print(f"   Found: {vault_path}")
            print(f"   Expected: {card_path}")
            print(f"🦫 TYRANNICAL RULING: Publish card to root first!")
            sys.exit(1)
        
        print(f"❌ Card not found: {card_path}")
        print(f"🦫 BEAVER SEARCH: Checked project root for active card")
        print(f"   (Cards in dam vault are LOCKED until published)")
        sys.exit(1)
    
    # BEAVER WORKFLOW ENFORCEMENT: Check which card is currently active
    current_cards = find_current_card()
    
    # Load target card
    card_data = load_card(card_path)
    current_status = card_data.get('status', '🟡 CURRENT')
    
    # TYRANNICAL WORKFLOW RULES
    if len(current_cards) == 0 and current_status != '🟡 CURRENT':
        print(f"🦫 BEAVER CONFUSION: No current card found!")
        print(f"   Card {card_number:03d} status: {current_status}")
        print(f"🦫 WORKFLOW ERROR: Cannot update non-current card when no card is active!")
        sys.exit(1)
    
    elif len(current_cards) > 1:
        print(f"🦫 BEAVER PANIC: Multiple current cards detected!")
        for card_num, _ in current_cards:
            print(f"   Card {card_num:03d}: 🟡 CURRENT")
        print(f"🦫 TYRANNICAL RULING: Fix workflow corruption first!")
        sys.exit(1)
    
    elif len(current_cards) == 1:
        current_card_num = current_cards[0][0]
        if card_number != current_card_num and current_status != '🟡 CURRENT':
            print(f"🦫 BEAVER WORKFLOW RAGE: Card {current_card_num:03d} is CURRENT!")
            print(f"   You're trying to update Card {card_number:03d} (status: {current_status})")
            print(f"🦫 TYRANNICAL RULING: Complete current card first!")
            print(f"   Allowed: Update Card {current_card_num:03d} or make Card {card_number:03d} current")
            sys.exit(1)
    
    # Validate new status
    if new_status not in VALID_STATUSES:
        print(f"❌ Invalid status: {new_status}")
        print(f"Valid statuses: {list(VALID_STATUSES.keys())}")
        sys.exit(1)
    
    # Check valid transition
    valid_next = VALID_TRANSITIONS.get(current_status, [])
    if new_status not in valid_next and new_status != current_status:
        print(f"❌ Invalid transition: {current_status} → {new_status}")
        print(f"Valid next statuses: {valid_next}")
        sys.exit(1)
    
    # Update status
    card_data['status'] = new_status
    save_card(card_path, card_data)
    
    # Success message
    status_emoji = new_status.split()[0]
    print(f"🦫 BEAVER UPDATE: Card {card_number:03d} → {status_emoji}")
    print(f"   {current_status} → {new_status}")
    
    return True

def publish_card(card_number):
    """Publish card from fortress vault to project root"""
    
    # Look for card in fortress vault
    fortress_pattern = Path(f".dams/fortress/work_cards/CARD_{card_number:03d}_*.yml")
    fortress_cards = list(Path(".").glob(str(fortress_pattern)))
    
    if not fortress_cards:
        print(f"❌ Card {card_number:03d} not found in fortress vault")
        print(f"🦫 BEAVER SEARCH: Checked .dams/fortress/work_cards/")
        return False
    
    if len(fortress_cards) > 1:
        print(f"🦫 BEAVER CONFUSION: Multiple cards found for {card_number:03d}")
        for card in fortress_cards:
            print(f"   - {card.name}")
        return False
    
    # Check if already published
    published_card = Path(f"CARD_{card_number:03d}.yml")
    if published_card.exists():
        print(f"🦫 BEAVER RAGE: Card {card_number:03d} already published!")
        print(f"   Published card: {published_card}")
        print(f"🦫 TYRANNICAL RULING: Archive existing card first!")
        return False
    
    # Load source card from fortress
    source_card = fortress_cards[0]
    card_data = load_card(source_card)
    
    # Set initial status for published card
    card_data['status'] = '🟡 CURRENT'
    card_data['card_number'] = f'{card_number:03d}'  # Normalize card number format
    
    # Publish to project root
    save_card(published_card, card_data)
    
    print(f"🦫 BEAVER PUBLISH SUCCESS: Card {card_number:03d} → Project Root")
    print(f"   Source: {source_card}")
    print(f"   Published: {published_card}")
    print(f"   Status: 🟡 CURRENT (ready for TDD work)")
    
    return True

def show_usage():
    """Show script usage"""
    print("🦫 Teddy's Card Status Manager")
    print("\nUsage:")
    print("  python3 update_card_status.py <command> [args]")
    print("\nCommands:")
    print("  publish <card_number>              - Publish card from vault to project root")
    print("  update <card_number> <new_status> - Update published card status")
    print("\nValid Statuses:")
    for status, desc in VALID_STATUSES.items():
        print(f"  {status} - {desc}")
    print("\nExamples:")
    print("  python3 update_card_status.py publish 1")
    print("  python3 update_card_status.py update 1 '🔴 RED'")
    print("  python3 update_card_status.py update 1 '🟢 GREEN'")
    print("  python3 update_card_status.py update 1 '✅ APPROVED'")

if __name__ == "__main__":
    if len(sys.argv) < 3:
        show_usage()
        sys.exit(1)
    
    command = sys.argv[1].lower()
    
    try:
        if command == "publish":
            if len(sys.argv) != 3:
                print("❌ Usage: publish <card_number>")
                sys.exit(1)
            card_number = int(sys.argv[2])
            publish_card(card_number)
            
        elif command == "update":
            if len(sys.argv) != 4:
                print("❌ Usage: update <card_number> <new_status>")
                sys.exit(1)
            card_number = int(sys.argv[2])
            new_status = sys.argv[3]
            update_status(card_number, new_status)
            
        else:
            print(f"❌ Unknown command: {command}")
            print("🦫 BEAVER CONFUSION: Use 'publish' or 'update'")
            show_usage()
            sys.exit(1)
            
    except ValueError:
        print("❌ Card number must be an integer")
        sys.exit(1)
    except KeyboardInterrupt:
        print("\n🦫 Beaver interrupted!")
        sys.exit(1)