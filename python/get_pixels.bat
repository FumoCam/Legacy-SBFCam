@echo off
::cv_get_backpack_hover
poetry run python cv_functions.py --func detect_pixels --left 691 --top 669 --width 9 --height 5 --detect_value 179,179,179 --debug true
::cv_get_navbar
poetry run python cv_functions.py --func detect_pixels --left 691 --top 669 --width 9 --height 5 --detect_value 255,255,255 --debug true
::cv_get_navbar_hidden (should be false)
poetry run python cv_functions.py --func detect_pixels --left 691 --top 669 --width 9 --height 5 --detect_value 255,255,255 --debug true
