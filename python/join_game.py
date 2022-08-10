from argparse import ArgumentParser
from json import dump as json_dump
from json import load as json_load
from pathlib import Path
from subprocess import call as call_proc  # nosec
from time import sleep
from traceback import format_exc

from psutil import process_iter
from selenium import webdriver
from selenium.webdriver.chrome.service import Service

BROWSER_EXECUTABLE_NAME = "chrome.exe"
SELENIUM_PATH = Path("../", "resources", "selenium")
BROWSER_PROFILE_PATH = Path(SELENIUM_PATH, ".browser_profile")
COOKIES_PATH = Path(SELENIUM_PATH, "cookies.json")


def is_process_running(name: str) -> bool:
    return len([proc for proc in process_iter() if proc.name() == name]) > 0


def kill_process(executable: str, force: bool = False):
    # TODO: taskkill.exe can fail, how can we kill the thing that should kill?
    # https://i.imgur.com/jd01ZOv.png
    process_call = ["taskkill"]
    if force:
        process_call.append("/F")
    process_call.append("/IM")
    process_call.append(executable)
    call_proc(process_call)  # nosec


def launch_roblox(
    game_id: str,
    instance_id: str,
    driver_executable: str,
) -> bool:
    js_code = f'Roblox.GameLauncher.joinGameInstance({game_id}, "{instance_id}")'
    DRIVER_PATH = Path(SELENIUM_PATH, driver_executable)
    DESIRED_URL = "https://roblox.com/"
    EXPECTED_EXECUTABLE = "RobloxPlayerBeta.exe"
    try:
        with open(COOKIES_PATH, "r", encoding="utf-8") as f:
            cookies = json_load(f)
    except FileNotFoundError:
        print("COOKIES PATH NOT FOUND.\nRun this with '--mode cookies' first.")
        return False

    options = webdriver.ChromeOptions()

    BROWSER_PROFILE_PATH.mkdir(parents=True, exist_ok=True)
    options.add_argument(f"--user-data-dir={BROWSER_PROFILE_PATH.resolve()}")
    driver_service = Service(DRIVER_PATH)
    driver = webdriver.Chrome(options=options, service=driver_service)
    driver.get(DESIRED_URL)

    for cookie in cookies:
        try:
            driver.add_cookie(cookie)
        except Exception:
            print(f"ERROR ADDING COOKIE: \n{cookie}\n")

    driver.refresh()
    driver.execute_script(js_code)

    CHECK_INTERVAL = 0.25
    MAX_SECONDS_TO_WAIT = 30
    success = False
    for _ in range(int(MAX_SECONDS_TO_WAIT / CHECK_INTERVAL)):
        if is_process_running(EXPECTED_EXECUTABLE):
            success = True
            break
        sleep(CHECK_INTERVAL)
    try:
        driver.quit()
        kill_process(BROWSER_EXECUTABLE_NAME)
        kill_process(driver_executable)
    except Exception:
        print(format_exc())

    if not success:
        raise Exception("Failure to launch.")

    return True


def save_cookies(driver_executable: str):
    DRIVER_PATH = Path(SELENIUM_PATH, driver_executable)

    print(
        "Login to your account in the brower that opens, come back to this screen, and press enter."
    )
    print("Press enter to start")
    input()

    driver_service = Service(DRIVER_PATH)
    driver = webdriver.Chrome(service=driver_service)
    driver.get("https://www.roblox.com/login")

    input("\n\n\nPress Enter to save cookies\n\n\n\n")

    with open(COOKIES_PATH, "w", encoding="utf-8") as f:
        json_dump(driver.get_cookies(), f, ensure_ascii=False, indent=4)

    driver.quit()
    kill_process(driver_executable)
    kill_process(BROWSER_EXECUTABLE_NAME)

    print(f"\n\n\nCookies saved to {COOKIES_PATH}, DO NOT SHARE THIS WITH ANYONE.")
    input("\nPress Enter to close.")


def main():
    VALID_MODES = ["launch", "cookies"]
    parser = ArgumentParser("Selenium Roblox Game Joiner")
    parser.add_argument(
        "--mode",
        help="Can be either 'cookies' to set cookies, or 'launch' to launch game.",
        type=str,
        default="launch",
    )
    parser.add_argument(
        "--game",
        help="Game ID of the Roblox game to join. Found in URL. Required for launch.",
        type=int,
    )
    parser.add_argument(
        "--instance",
        help="Instance ID of the Roblox game to join. Found by API. Required for launch.",
        type=str,
    )
    parser.add_argument(
        "--driver",
        help="Executable name of the desired Selenium driver.",
        type=str,
        default="chromedriver.exe",
    )
    args = parser.parse_args()

    if args.mode == "launch":
        if not args.game:
            raise ValueError("Expected value for arg 'game'.")
        if not args.instance:
            raise ValueError("Expected value for arg 'instance'.")

        launch_roblox(args.game, args.instance, args.driver)
    elif args.mode == "cookies":
        save_cookies(args.driver)
    else:
        raise ValueError(
            f"Invalid option '{args.mode}'. Valid options are [{','.join(VALID_MODES)}]."
        )


if __name__ == "__main__":
    main()
