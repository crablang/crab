#!/bin/bash
# A quick smoke test to make sure publish_tooolstate.py works.

set -euo pipefail
IFS=$'\n\t'

rm -rf crablang-toolstate
git clone --depth=1 https://github.com/crablang-nursery/crablang-toolstate.git
cd crablang-toolstate
python3 "../../src/tools/publish_toolstate.py" "$(git rev-parse HEAD)" \
    "$(git log --format=%s -n1 HEAD)" "" ""
cd ..
rm -rf crablang-toolstate
