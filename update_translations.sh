#!/bin/sh

echo "Generating cemcl.pot ..."
cd res/ui
find -name \*.slint | xargs slint-tr-extractor -o ../translation/cemcl.pot

echo "Updating zh.po ..."
cd ../translation/zh/LC_MESSAGES/
msgmerge cemcl.po ../../cemcl.pot -o cemcl.po
