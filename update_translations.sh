#!/bin/sh

echo "Generating cemcl.pot ..."
cd res/ui
find -name \*.slint | xargs slint-tr-extractor -o ../translation/cemcl.pot

echo "Updating zh.po ..."
cd ../translation
msgmerge zh.po cemcl.pot -o zh.po

echo "Generating zh.mo ..."
msgfmt zh.po -o zh.mo

if [ -z $1 ]
then
    echo "Moving zh.mo to target/debug/locale/cemcl.mo ..."
    rm -rf ../../target/debug/locale
    mkdir -p ../../target/debug/locale/zh/LC_MESSAGES
    mv zh.mo ../../target/debug/locale/zh/LC_MESSAGES/cemcl.mo
elif [ $1 = "--release" ]
then
    echo "Moving zh.mo to target/release/locale/zh/LC_MESSAGES/cemcl.mo ..."
    rm -rf ../../target/release/locale
    mkdir -p ../../target/release/locale/zh/LC_MESSAGES
    mv zh.mo ../../target/release/locale/zh/LC_MESSAGES/cemcl.mo
elif [ $1 = "--all" ]
then
    echo "Copying zh.mo to target/debug/locale/zh/LC_MESSAGES/cemcl.mo ..."
    rm -rf ../../target/debug/locale
    mkdir -p ../../target/debug/locale/zh/LC_MESSAGES
    cp zh.mo ../../target/debug/locale/zh/LC_MESSAGES/cemcl.mo

    echo "Moving zh.mo to target/release/locale/zh/LC_MESSAGES/cemcl.mo ..."
    rm -rf ../../target/release/locale
    mkdir -p ../../target/release/locale/zh/LC_MESSAGES
    mv zh.mo ../../target/release/locale/zh/LC_MESSAGES/cemcl.mo
fi

echo "Done."
