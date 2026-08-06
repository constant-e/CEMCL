#!/bin/sh

echo "Generating frontend.pot ..."
cd res/frontend
find -name \*.slint | xargs slint-tr-extractor -o ../translation/frontend.pot

echo "Updating zh_CN.po ..."
cd ../translation/zh_CN/LC_MESSAGES
msgmerge frontend.po ../../frontend.pot -o frontend.po
