import json
from argparse import ArgumentParser
from typing import List, TypedDict

import cv2
import numpy as np
from mss import mss


class CaptureRegion(TypedDict):
    left: int
    top: int
    width: int
    height: int


def get_screenshot(capture_region: CaptureRegion) -> np.ndarray:
    with mss() as sct:
        return np.array(sct.grab(capture_region))


def compare_thing(source, comparator):
    # TODO: Whats the way to do this in numpy? Some np.all function? np.where?
    for row in source:
        for pixel in row:
            if not np.array_equal(pixel, comparator):
                return False
    return True


def detect_pixels(
    capture_region: CaptureRegion, detect_value: List[int], debug: bool
) -> bool:
    if debug:
        from time import sleep

        sleep(2)

    screenshot = get_screenshot(capture_region)
    pixels_np = np.array(detect_value + [255])

    pixels_match = compare_thing(screenshot, pixels_np)

    if debug:
        for row in screenshot:
            print("row")
            for pixel in row:
                print(pixel)
        cv2.imshow(
            "Pixel Detection Capture",
            cv2.resize(screenshot, None, fx=3, fy=3, interpolation=cv2.INTER_CUBIC),
        )
        cv2.waitKey(0)
    return pixels_match


def main():
    VALID_FUNCS = ["detect_pixels"]
    parser = ArgumentParser("OpenCV Functions")
    parser.add_argument(
        "--func",
        help="Which function to execute",
        type=str,
        required=True,
    )
    parser.add_argument(
        "--left",
        help="Left pixel of capture region",
        type=int,
        default=0,
    )
    parser.add_argument(
        "--top",
        help="Top pixel of capture region",
        type=int,
        default=0,
    )
    parser.add_argument(
        "--width",
        help="Amount of pixels right to include in capture region",
        type=int,
        default=1280,
    )
    parser.add_argument(
        "--height",
        help="Amount of pixels down to include in capture region",
        type=int,
        default=720,
    )
    parser.add_argument(
        "--detect_value",
        help="RGB value to detect, i.e. '255,255,255'",
        type=str,
    )
    parser.add_argument(
        "--debug",
        help="Whether to enable debug data, useful for development",
        type=bool,
        default=False,
    )
    args = parser.parse_args()

    if args.func == "detect_pixels":
        if not args.detect_value:
            raise ValueError("Missing option '--detect_value'")

        detection_arg = args.detect_value.split(",")
        if len(detection_arg) != 3:
            raise ValueError(
                f"Invalid option '{args.detect_value}', must be exactly three options"
            )
        detection_val = []
        for number_str in detection_arg:
            if not number_str.isnumeric():
                raise ValueError(
                    f"Bad sub-value '{number_str}' for option '{args.detect_value}', values must be numeric"
                )
            num = int(number_str)
            if num > 255 or num < 0:
                raise ValueError(
                    f"Bad sub-value '{num}' for option '{args.detect_value}', values must be between 0-255"
                )
            detection_val.append(num)

        capture_area = {
            "left": args.left,
            "top": args.top,
            "width": args.width,
            "height": args.height,
        }

        result = detect_pixels(capture_area, detection_val, args.debug)
        print(json.dumps(result))
    else:
        raise ValueError(
            f"Invalid option '{args.func}'. Valid options are [{','.join(VALID_FUNCS)}]."
        )


if __name__ == "__main__":
    # poetry run python .\cv_functions.py --func detect_pixels --left 690 --top 669 --width 10 --height 5 --detect_value 173,173,173 --debug true
    # poetry run python .\cv_functions.py --func detect_pixels --left 690 --top 669 --width 10 --height 5 --detect_value 246,246,246 --debug true
    main()
