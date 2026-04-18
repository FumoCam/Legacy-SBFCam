
from typing import Tuple

from requests import get


def log(msg):
    print(msg)

game_id = 7363647365

def get_player_tokens() -> Tuple[bool, str]:
    log("Querying Roblox API for server list")
    url = f"https://games.roblox.com/v1/games/{game_id}/servers/Public"
    try:
        response = get(url, timeout=10)
    except Exception:
        return False, "[WARN] Could not poll Roblox servers. Is Roblox down?"
    if response.status_code == 200:
        log("Finding best server and comparing to current...")

        try:
            response_result = response.json()
        except:
            return False, "[WARN] Failed to parse Roblox server JSON. Is Roblox malfunctioning?"

        servers = response_result["data"]
        print(servers)

if __name__ == "__main__":
    get_player_tokens()
