#!/bin/bash

set -e

cargo b
cp target/debug/libreaper_touchbar.dylib ~/Library/Application\ Support/REAPER/UserPlugins/reaper_touchbar.dylib

echo 'done :)'
