import os
import zipfile
import datetime
import json
import base64

def base64_encode(data):
    """Encode bytes to base64 string"""
    return base64.b64encode(data).decode()

def zip_milestone_cards():
    fortress_path = "/home/xnull/repos/code/rust/oodx/prontodb/.dams/fortress"
    work_cards_path = os.path.join(fortress_path, "work_cards")
    milestone_vaults_path = os.path.join(fortress_path, "milestone_vaults")
    
    os.makedirs(milestone_vaults_path, exist_ok=True)
    
    milestones = [
        {"name": "MILESTONE_2_CORE_KV_OPS", "card_pattern": "CARD_00[6-9].yml,CARD_01[0-2].yml"},
        {"name": "MILESTONE_3_TTL_CACHE", "card_pattern": "CARD_01[3-9].yml"},
        {"name": "MILESTONE_4_SECURITY", "card_pattern": "CARD_02[1-8].yml"},
        {"name": "MILESTONE_5_DATA_MGMT", "card_pattern": "CARD_02[9-35].yml"},
        {"name": "MILESTONE_6_FILESYSTEM_MIRROR", "card_pattern": "CARD_03[6-42].yml"}
    ]
    
    password_vault = []
    
    for milestone in milestones:
        zip_filename = f"{milestone['name']}.zip"
        zip_path = os.path.join(milestone_vaults_path, zip_filename)
        
        # Generate a unique password
        timestamp = datetime.datetime.now().strftime("%Y%m%d")
        password = f"beaver_dam_{timestamp}_{milestone['name']}"
        
        # Create zip file
        with zipfile.ZipFile(zip_path, 'w', zipfile.ZIP_DEFLATED) as zipf:
            for filename in os.listdir(work_cards_path):
                if any(pattern in filename for pattern in milestone['card_pattern'].split(',')):
                    card_path = os.path.join(work_cards_path, filename)
                    zipf.write(card_path, arcname=filename)
        
        # Store password details
        password_vault.append({
            "milestone": milestone['name'],
            "password": password
        })
    
    return password_vault

def save_password_vault(password_vault):
    """Save password vault to a secure location"""
    # Use the actual environment variable for secure storage
    agentic_etc = os.environ.get('AGENTIC_ETC', '/tmp/agentic_etc')
    os.makedirs(agentic_etc, exist_ok=True)
    
    vault_path = os.path.join(agentic_etc, "beaver_passwords.secure")
    
    with open(vault_path, 'w') as f:
        json.dump(password_vault, f, indent=2)
    
    # Set restrictive permissions
    os.chmod(vault_path, 0o600)

if __name__ == "__main__":
    password_vault = zip_milestone_cards()
    save_password_vault(password_vault)
    print("Milestone cards zipped and passwords secured!")