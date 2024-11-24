#!/bin/sh

echo "Generating cemcl.pot ..."
cd res/ui
find -name \*.slint | xargs slint-tr-extractor -o ../translation/cemcl.pot

echo "Updating zh.po ..."
cd ../translation
msgmerge zh.po cemcl.pot -o zh.po
