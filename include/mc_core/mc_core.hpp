#pragma once

#include <iostream>
#include <vector>

#include "sonic/sonic.h"

#define DEFAULTACC "[{\"online\":false,\"userName\":\"Steve\",\"uuid\":\"\",\"token\":\"\"}]"

using sonic_json::Node;
using std::string;
using std::vector;
// 已有变量：gameDir - .minecraft位置    javaDir - java位置(可执行文件)

// 账户
struct Account {
    bool online;        // 在线模式
    string token;       // accessToken
    string userName;    // 用户名
    string uuid;        // UUID
};

// 一个MC版本
struct Game {
    string args;    // 自定义参数（没有留空）
    int height;     // 窗口高度（留空 cfg.height > 默认）
    string type;    // 类型
    string version; // 版本
    int width;      // 窗口宽度（留空 cfg.width > 默认）
    string xms;     // 最小jvm内存（留空 cfg.xms > 默认）
    string xmx;     // 最大jvm内存（留空 cfg.xmx > 默认）
};

string addArgs(Node & n);
bool addGame();
string addLibs(Node & n, string gameDir);
Account addOfflineAccount();
Account addOnlineAccount();
bool delGame();
string getCMD(Account account, Game game, string javaDir, string gameDir);
vector<Game> loadGameList(bool reload, string gameDir, int defHeight, int defWidth, string defXms, string defXmx);
