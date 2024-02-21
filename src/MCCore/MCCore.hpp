#pragma once

#include <iostream>
#include <vector>

#include "sonic/sonic.h"

#define DEFAULTACC "[{\"type\":\"Legacy\",\"userName\":\"Steve\",\"uuid\":\"Please Enter UUID\",\"token\":\"None\"}]"

using sonic_json::Node;
using std::string;
using std::vector;

// 已有变量：gameDir - .minecraft位置    javaDir - java位置(可执行文件)

// 账户
struct Account {
    string type;        // 登录模式
    string token;       // accessToken
    string userName;    // 用户名
    string uuid;        // UUID
};

// 一个MC版本 留空/-1项使用默认
struct Game {
    string args = "";       // 自定义参数（没有留空）
    int height = -1;        // 窗口高度（默认 cfg.height）
    string javaDir = "";    // Java路径（默认 cfg.javaDir）
    bool seperated = false; // 版本隔离
    string type = "";       // 类型
    string version = "";    // 版本
    int width = -1;         // 窗口宽度（默认 cfg.width）
    string xms = "";        // 最小jvm内存（默认 cfg.xms）
    string xmx = "";        // 最大jvm内存（默认 cfg.xmx > 默认）
};

string addArgs(Node &n);
bool addGame();
vector<string> addLibs(Node &n, string gameDir);
Account addOfflineAccount();
Account addOnlineAccount();
bool delGame();
string getConfig(vector<Game> gameList, string javaDir, int defHeight, int defWidth, string defXms, string defXmx);
string getCMD(Account account, Game game, string javaDir, string gameDir);
vector<Game> loadGameList(bool reload, string gameDir, string javaDir, int defHeight, int defWidth, string defXms, string defXmx);
bool setArgs(Node &n, string *jvmArg, string *gameArg);
