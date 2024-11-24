#!/bin/sh

echo "Generating zh.mo ..."
cd res/translation
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
