#!/bin/sh

echo "Generating cemcl.pot ..."
cd res/ui
find -name \*.slint | xargs slint-tr-extractor -o ../translation/cemcl.pot

echo "Updating zh_CN.po ..."
cd ../translation/zh_CN/LC_MESSAGES
msgmerge cemcl.po ../../cemcl.pot -o cemcl.po
